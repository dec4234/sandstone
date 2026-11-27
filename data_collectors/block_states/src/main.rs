//! Get block state IDs from the vanilla server and write them as Rust constants.
//!
//! Two steps:
//! - `report <version>` downloads the vanilla server jar for `<version>` from Mojang and runs its data
//!   generator. The generator writes `generated/reports/blocks.json` into `work/<version>/`.
//! - `generate <blocks.json> <version>` reads that report and writes
//!   `sandstone/src/protocol/game/world/block_states.rs`. It also reads `registries.json` from the same
//!   folder and writes every block state, block entity type, item and entity type to
//!   `sandstone/src/protocol/game/world/block_state_table.rs`.
//!
//! The tool does not depend on sandstone. If the generated file does not compile, you can still run
//! this tool to write it again.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// The file that `generate` writes, relative to this crate.
const OUTPUT_FILE: &str = "../../sandstone/src/protocol/game/world/block_states.rs";

/// The file with every block state and every block entity type, relative to this crate.
const TABLE_FILE: &str = "../../sandstone/src/protocol/game/world/block_state_table.rs";

const USAGE: &str = "usage:
  cargo run -p block_states -- report <version>                 (example: report 26.3)
  cargo run -p block_states -- generate <blocks.json> <version>";

fn main() -> Result<(), Box<dyn Error>> {
	let args: Vec<String> = std::env::args().skip(1).collect();
	let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

	match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
		["report", version] => {
			let report = run_report(&crate_dir.join("work").join(version), version)?;
			println!("report written to {}", report.display());
		}
		["generate", report, version] => {
			let json = fs::read_to_string(report)?;
			let source = generate(&json, version)?;
			let out = crate_dir.join(OUTPUT_FILE);
			fs::write(&out, source)?;
			println!("constants written to {}", out.display());

			// The data generator writes registries.json next to blocks.json.
			let registries = fs::read_to_string(Path::new(report).with_file_name("registries.json"))?;
			let mut table = generate_table(&json, &registries, version)?;
			// The data generator writes the default components of each item into
			// `minecraft/components/item/<name>.json`, next to registries.json.
			let components_dir = Path::new(report).with_file_name("minecraft").join("components").join("item");
			let items = registry_entries(&registries, "minecraft:item")?;
			let components = items
				.iter()
				.map(|name| {
					let file = components_dir.join(format!("{}.json", name.trim_start_matches("minecraft:")));
					fs::read_to_string(&file).map_err(|e| format!("cannot read {}: {e}", file.display()).into())
				})
				.collect::<Result<Vec<String>, Box<dyn Error>>>()?;
			table.push_str(&generate_item_tables(&components)?);
			let out = crate_dir.join(TABLE_FILE);
			fs::write(&out, table)?;
			println!("state table written to {}", out.display());
		}
		_ => {
			eprintln!("{USAGE}");
			std::process::exit(2);
		}
	}
	Ok(())
}

/// Downloads the server jar for `version` (if it is not already in `work_dir`) and runs the vanilla
/// data generator in `work_dir`. Returns the path of the block report.
fn run_report(work_dir: &Path, version: &str) -> Result<PathBuf, Box<dyn Error>> {
	fs::create_dir_all(work_dir)?;
	let jar = work_dir.join("server.jar");

	if !jar.exists() {
		let manifest: serde_json::Value = serde_json::from_str(&get(VERSION_MANIFEST)?)?;
		let version_url = manifest["versions"]
			.as_array()
			.ok_or("version manifest has no `versions` array")?
			.iter()
			.find(|v| v["id"] == version)
			.and_then(|v| v["url"].as_str())
			.ok_or_else(|| format!("version `{version}` is not in the Mojang version manifest"))?;

		let version_json: serde_json::Value = serde_json::from_str(&get(version_url)?)?;
		let jar_url = version_json["downloads"]["server"]["url"].as_str().ok_or_else(|| format!("version `{version}` has no server download"))?;

		println!("downloading {jar_url}");
		let bytes = ureq::get(jar_url).call()?.body_mut().with_config().limit(512 * 1024 * 1024).read_to_vec()?;
		fs::write(&jar, bytes)?;
	}

	// The jar is a bundler. `bundlerMainClass` makes it start the data generator instead of the
	// server. It extracts its libraries into the current directory, so run it inside `work_dir`.
	println!("running the data generator (this can take a minute)");
	let status = Command::new("java")
		.current_dir(work_dir)
		.args(["-DbundlerMainClass=net.minecraft.data.Main", "-jar", "server.jar", "--reports"])
		.status()
		.map_err(|e| format!("could not start `java` (is it on PATH?): {e}"))?;
	if !status.success() {
		return Err(format!("data generator failed with {status}").into());
	}

	let report = work_dir.join("generated").join("reports").join("blocks.json");
	if !report.exists() {
		return Err(format!("data generator finished but {} does not exist", report.display()).into());
	}
	Ok(report)
}

fn get(url: &str) -> Result<String, Box<dyn Error>> {
	Ok(ureq::get(url).call()?.body_mut().read_to_string()?)
}

/// Turns the vanilla `blocks.json` report into Rust source with one constant per block. Each
/// constant is the block's default state ID. The constants are sorted by state ID.
fn generate(report_json: &str, version: &str) -> Result<String, Box<dyn Error>> {
	let report: serde_json::Map<String, serde_json::Value> = serde_json::from_str(report_json)?;

	let mut blocks: Vec<(String, i64)> = Vec::with_capacity(report.len());
	for (name, block) in &report {
		let states = block["states"].as_array().ok_or_else(|| format!("{name}: no `states` array"))?;
		let defaults: Vec<i64> = states.iter().filter(|s| s["default"] == true).filter_map(|s| s["id"].as_i64()).collect();
		let [default] = defaults.as_slice() else {
			return Err(format!("{name}: expected exactly one default state, found {}", defaults.len()).into());
		};
		blocks.push((const_name(name)?, *default));
	}
	blocks.sort_by_key(|(_, id)| *id);

	// Two names that map to the same constant would make the file not compile. Fail here instead.
	let mut seen = std::collections::HashSet::new();
	for (name, _) in &blocks {
		if !seen.insert(name) {
			return Err(format!("two blocks map to the constant name `{name}`").into());
		}
	}

	let mut out = format!(
		"//! @generated by `data_collectors/block_states` from the vanilla {version} server report. Do not edit.\n\
		 //!\n\
		 //! The default block state ID of each block. To write this file again, see\n\
		 //! `data_collectors/block_states/README.md`.\n\n"
	);
	for (name, id) in &blocks {
		out.push_str(&format!("pub const {name}: i32 = {id};\n"));
	}
	Ok(out)
}

/// Turns the vanilla `blocks.json` and `registries.json` reports into Rust source with every block
/// state and every block entity type.
///
/// The table keeps only the first state ID and the property values of each block. The other IDs
/// come from the vanilla order: the properties are sorted by name and the last property changes
/// fastest. This function checks that order against the ID of every state in the report. If the
/// order changes, the tool stops and does not write a table with wrong IDs.
fn generate_table(blocks_json: &str, registries_json: &str, version: &str) -> Result<String, Box<dyn Error>> {
	let report: serde_json::Map<String, serde_json::Value> = serde_json::from_str(blocks_json)?;

	struct Block {
		name: String,
		first_id: i64,
		count: i64,
		default_id: i64,
		properties: Vec<(String, Vec<String>)>,
	}

	let mut blocks = Vec::with_capacity(report.len());
	for (name, block) in &report {
		// serde_json keeps object keys sorted by name, which is the vanilla property order.
		let mut properties = Vec::new();
		if let Some(props) = block.get("properties") {
			let props = props.as_object().ok_or_else(|| format!("{name}: `properties` is not an object"))?;
			for (prop, values) in props {
				let values = values.as_array().ok_or_else(|| format!("{name}.{prop}: values are not an array"))?;
				let values = values
					.iter()
					.map(|v| v.as_str().map(str::to_string).ok_or_else(|| format!("{name}.{prop}: value is not a string")))
					.collect::<Result<Vec<_>, _>>()?;
				properties.push((prop.clone(), values));
			}
		}

		let states = block["states"].as_array().ok_or_else(|| format!("{name}: no `states` array"))?;
		let count: i64 = properties.iter().map(|(_, values)| values.len() as i64).product();
		if states.len() as i64 != count {
			return Err(format!("{name}: {} states, but the properties give {count}", states.len()).into());
		}

		let ids = states
			.iter()
			.map(|s| s["id"].as_i64().ok_or_else(|| format!("{name}: state without an `id`")))
			.collect::<Result<Vec<_>, _>>()?;
		let first_id = *ids.iter().min().ok_or_else(|| format!("{name}: no states"))?;
		let mut default_id = None;

		for (state, id) in states.iter().zip(&ids) {
			let mut index = 0i64;
			for (prop, values) in &properties {
				let value = state["properties"][prop].as_str().ok_or_else(|| format!("{name}: state {id} has no value for `{prop}`"))?;
				let position = values.iter().position(|v| v == value).ok_or_else(|| format!("{name}: state {id} has unknown value `{prop}={value}`"))?;
				index = index * values.len() as i64 + position as i64;
			}
			if first_id + index != *id {
				return Err(format!("{name}: state {id} is not at the expected ID {} (the state order changed)", first_id + index).into());
			}
			if state["default"] == true {
				if default_id.is_some() {
					return Err(format!("{name}: more than one default state").into());
				}
				default_id = Some(*id);
			}
		}

		blocks.push(Block {
			name: name.clone(),
			first_id,
			count,
			default_id: default_id.ok_or_else(|| format!("{name}: no default state"))?,
			properties,
		});
	}
	blocks.sort_by_key(|b| b.first_id);

	// A lookup by ID does a binary search over `first_id`, so the ranges must not have gaps.
	let mut next_id = 0;
	for block in &blocks {
		if block.first_id != next_id {
			return Err(format!("{}: first state ID is {}, expected {next_id}", block.name, block.first_id).into());
		}
		next_id += block.count;
	}

	let entity_types = registry_entries(registries_json, "minecraft:block_entity_type")?;
	let items = registry_entries(registries_json, "minecraft:item")?;
	let entities = registry_entries(registries_json, "minecraft:entity_type")?;

	let mut out = format!(
		"//! @generated by `data_collectors/block_states` from the vanilla {version} server reports. Do not edit.
//!
//! Every block state, every block entity type, every item and every entity type. To write this file again, see
//! `data_collectors/block_states/README.md`.

/// One block and the range of state IDs that it owns.
///
/// The state IDs of a block are `first_id..first_id + (product of the value counts)`. The index of a
/// state in that range comes from the index of each property value. The last property changes fastest.
#[derive(Debug)]
pub struct BlockInfo {{
	pub name: &'static str,
	pub first_id: i32,
	pub default_id: i32,
	/// `(property name, values)`, sorted by property name.
	pub properties: &'static [(&'static str, &'static [&'static str])],
}}

/// The number of block states. The direct palette of a chunk section uses `ceil(log2(BLOCK_STATE_COUNT))` bits.
pub const BLOCK_STATE_COUNT: i32 = {next_id};

/// Every block, sorted by `first_id`.
pub const BLOCKS: &[BlockInfo] = &[
"
	);
	for block in &blocks {
		let properties: Vec<String> = block
			.properties
			.iter()
			.map(|(prop, values)| format!("(\"{prop}\", &[{}])", values.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", ")))
			.collect();
		out.push_str(&format!(
			"\tBlockInfo {{ name: \"{}\", first_id: {}, default_id: {}, properties: &[{}] }},\n",
			block.name,
			block.first_id,
			block.default_id,
			properties.join(", ")
		));
	}
	out.push_str("];\n\n/// Block entity type names. The index of a name is its protocol ID.\npub const BLOCK_ENTITY_TYPES: &[&str] = &[\n");
	for name in entity_types {
		out.push_str(&format!("\t\"{name}\",\n"));
	}
	out.push_str("];\n\n/// Item names. The index of a name is its protocol ID (the `minecraft:item` registry).\npub const ITEMS: &[&str] = &[\n");
	for name in items {
		out.push_str(&format!("\t\"{name}\",\n"));
	}
	out.push_str("];\n\n/// Entity type names. The index of a name is its protocol ID (the `minecraft:entity_type` registry).\npub const ENTITY_TYPES: &[&str] = &[\n");
	for name in entities {
		out.push_str(&format!("\t\"{name}\",\n"));
	}
	out.push_str("];\n");
	Ok(out)
}

/// The entry names of one registry in `registries.json`, in protocol ID order. Fails if the IDs are
/// not exactly `0..n`.
fn registry_entries(registries_json: &str, registry: &str) -> Result<Vec<String>, Box<dyn Error>> {
	let report: serde_json::Value = serde_json::from_str(registries_json)?;
	let entries = report[registry]["entries"].as_object().ok_or_else(|| format!("registries.json has no `{registry}` entries"))?;

	let mut names = vec![None; entries.len()];
	for (name, entry) in entries {
		let id = entry["protocol_id"].as_u64().ok_or_else(|| format!("{registry} {name}: no `protocol_id`"))? as usize;
		let slot = names.get_mut(id).ok_or_else(|| format!("{registry} {name}: protocol ID {id} is out of range"))?;
		if slot.replace(name.clone()).is_some() {
			return Err(format!("{registry}: two entries have protocol ID {id}").into());
		}
	}
	Ok(names.into_iter().map(|n| n.expect("n entries with distinct IDs in 0..n fill every slot")).collect())
}

/// Turns the default components of each item (the JSON files of the components report, in
/// protocol ID order) into two tables: the default stack size of each item, and the equipment
/// slot of the items that have the `equippable` component.
fn generate_item_tables(components: &[String]) -> Result<String, Box<dyn Error>> {
	let mut sizes = Vec::with_capacity(components.len());
	let mut slots = Vec::new();
	for (id, json) in components.iter().enumerate() {
		let report: serde_json::Value = serde_json::from_str(json)?;
		let components = &report["components"];
		let size = components["minecraft:max_stack_size"].as_u64().ok_or_else(|| format!("item {id}: no `max_stack_size`"))?;
		sizes.push(u8::try_from(size).map_err(|_| format!("item {id}: stack size {size} is too large"))?);
		if let Some(slot) = components["minecraft:equippable"]["slot"].as_str() {
			slots.push((id, slot.to_string()));
		}
	}

	let mut out = String::from("
/// The default stack size of each item. The index is the item protocol ID.
pub const ITEM_MAX_STACK_SIZES: &[u8] = &[
");
	for chunk in sizes.chunks(32) {
		out.push_str(&format!("	{},
", chunk.iter().map(u8::to_string).collect::<Vec<_>>().join(", ")));
	}
	out.push_str("];

/// The equipment slot of the items that have the `equippable` component: `(item protocol ID, slot)`,
/// sorted by ID. The slot is `head`, `chest`, `legs`, `feet`, `body`, `saddle` or `offhand`.
pub const ITEM_EQUIPMENT_SLOTS: &[(i32, &str)] = &[
");
	for (id, slot) in slots {
		out.push_str(&format!("	({id}, \"{slot}\"),
"));
	}
	out.push_str("];
");
	Ok(out)
}

/// `minecraft:grass_block` -> `GRASS_BLOCK`.
fn const_name(id: &str) -> Result<String, Box<dyn Error>> {
	let path = id.strip_prefix("minecraft:").ok_or_else(|| format!("{id}: not in the `minecraft` namespace"))?;
	let name: String = path.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' }).collect();
	if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) {
		return Err(format!("{id}: cannot make a Rust constant name from it").into());
	}
	Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
	fn constant_is_the_default_state_not_the_first() {
		// grass_block has two states. The first one (snowy=true) is not the default. A superflat
		// world built from the first state would show snowy grass.
		let report = r#"{
			"minecraft:grass_block": { "states": [ { "id": 8 }, { "id": 9, "default": true } ] },
			"minecraft:air": { "states": [ { "id": 0, "default": true } ] }
		}"#;
		let out = generate(report, "test").unwrap();
		assert!(out.contains("pub const GRASS_BLOCK: i32 = 9;\n"), "{out}");
		// Sorted by state ID, so air comes first.
		assert!(out.find("AIR").unwrap() < out.find("GRASS_BLOCK").unwrap());
	}

	#[test]
	fn block_without_one_default_is_an_error() {
		// If the report format changes and `default` goes away, the tool must stop, not write a
		// file with wrong IDs.
		assert!(generate(r#"{ "minecraft:stone": { "states": [ { "id": 1 } ] } }"#, "test").is_err());
		assert!(generate(r#"{ "minecraft:stone": { "states": [ { "id": 1, "default": true }, { "id": 2, "default": true } ] } }"#, "test").is_err());
	}

	const NO_ENTITIES: &str = r#"{ "minecraft:block_entity_type": { "entries": {} }, "minecraft:item": { "entries": {} }, "minecraft:entity_type": { "entries": {} } }"#;

	#[test]
	fn table_keeps_the_vanilla_state_order() {
		// Properties sorted by name, the last one changes fastest: a=0,b=1 is first_id + 1.
		let report = r#"{
			"minecraft:air": { "states": [ { "id": 0, "default": true } ] },
			"minecraft:x": { "properties": { "b": ["0", "1"], "a": ["0", "1"] }, "states": [
				{ "id": 1, "properties": { "a": "0", "b": "0" } },
				{ "id": 2, "properties": { "a": "0", "b": "1" }, "default": true },
				{ "id": 3, "properties": { "a": "1", "b": "0" } },
				{ "id": 4, "properties": { "a": "1", "b": "1" } }
			] }
		}"#;
		let out = generate_table(report, NO_ENTITIES, "test").unwrap();
		assert!(
			out.contains(r#"BlockInfo { name: "minecraft:x", first_id: 1, default_id: 2, properties: &[("a", &["0", "1"]), ("b", &["0", "1"])] },"#),
			"{out}"
		);
		assert!(out.contains("pub const BLOCK_STATE_COUNT: i32 = 5;"), "{out}");
	}

	#[test]
	fn table_refuses_a_changed_state_order() {
		// Here the first property changes fastest. IDs computed from the table would be wrong.
		let report = r#"{
			"minecraft:x": { "properties": { "a": ["0", "1"], "b": ["0", "1"] }, "states": [
				{ "id": 0, "properties": { "a": "0", "b": "0" }, "default": true },
				{ "id": 1, "properties": { "a": "1", "b": "0" } },
				{ "id": 2, "properties": { "a": "0", "b": "1" } },
				{ "id": 3, "properties": { "a": "1", "b": "1" } }
			] }
		}"#;
		assert!(generate_table(report, NO_ENTITIES, "test").is_err());
	}

	#[test]
	fn table_refuses_a_gap_in_the_ids() {
		let report = r#"{
			"minecraft:air": { "states": [ { "id": 0, "default": true } ] },
			"minecraft:stone": { "states": [ { "id": 2, "default": true } ] }
		}"#;
		assert!(generate_table(report, NO_ENTITIES, "test").is_err());
	}

	#[test]
	fn block_entity_types_are_in_protocol_id_order() {
		let registries = r#"{ "minecraft:block_entity_type": { "entries": { "minecraft:a": { "protocol_id": 1 }, "minecraft:b": { "protocol_id": 0 } } } }"#;
		assert_eq!(registry_entries(registries, "minecraft:block_entity_type").unwrap(), vec!["minecraft:b", "minecraft:a"]);

		let gap = r#"{ "minecraft:block_entity_type": { "entries": { "minecraft:a": { "protocol_id": 1 } } } }"#;
		assert!(registry_entries(gap, "minecraft:block_entity_type").is_err());
	}

	#[test]
	fn item_tables_keep_the_id_order() {
		let items = [
			r#"{"components": {"minecraft:max_stack_size": 64}}"#.to_string(),
			r#"{"components": {"minecraft:max_stack_size": 1, "minecraft:equippable": {"slot": "head"}}}"#.to_string(),
		];
		let out = generate_item_tables(&items).unwrap();
		assert!(out.contains("pub const ITEM_MAX_STACK_SIZES: &[u8] = &[
	64, 1,
];"));
		assert!(out.contains("	(1, \"head\"),
];"));
		assert!(generate_item_tables(&[r#"{"components": {}}"#.to_string()]).is_err(), "a missing stack size is an error");
	}

	#[test]
	fn names_that_collide_are_an_error() {
		let report = r#"{
			"minecraft:a_b": { "states": [ { "id": 0, "default": true } ] },
			"minecraft:a.b": { "states": [ { "id": 1, "default": true } ] }
		}"#;
		assert!(generate(report, "test").is_err());
	}
}
