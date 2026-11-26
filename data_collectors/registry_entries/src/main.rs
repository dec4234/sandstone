//! Get the entry IDs of each synchronized registry and every network tag from the vanilla data pack,
//! and write them as Rust constants.
//!
//! `generate <data/minecraft dir> <registries.json> <version>` reads the file names in
//! `<dir>/<registry>/`, the tag files in `<dir>/tags/`, and the built-in registry IDs in the vanilla
//! `registries.json` report. It writes `sandstone/src/protocol/game/info/registry/vanilla_registries.rs`.
//!
//! The tool does not depend on sandstone. If the generated file does not compile, you can still run
//! this tool to write it again.

#![allow(clippy::type_complexity)]

use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// The file that `generate` writes, relative to this crate.
const OUTPUT_FILE: &str = "../../sandstone/src/protocol/game/info/registry/vanilla_registries.rs";

/// The registries that the server sends to the client in Registry Data packets (26.3).
/// <https://minecraft.wiki/w/Java_Edition_protocol/Registries>. The wiki list does not include
/// `decorated_pot_pattern`, but the 26.3 client disconnects without it.
const SYNCED_REGISTRIES: [&str; 31] = [
	"banner_pattern",
	"block_transformer",
	"cat_sound_variant",
	"cat_variant",
	"chat_type",
	"chicken_sound_variant",
	"chicken_variant",
	"cow_sound_variant",
	"cow_variant",
	"damage_type",
	"decorated_pot_pattern",
	"dialog",
	"dimension_type",
	"enchantment",
	"frog_variant",
	"instrument",
	"jukebox_song",
	"painting_variant",
	"pig_sound_variant",
	"pig_variant",
	"sulfur_cube_archetype",
	"test_environment",
	"test_instance",
	"timeline",
	"trim_material",
	"trim_pattern",
	"wolf_sound_variant",
	"wolf_variant",
	"world_clock",
	"worldgen/biome",
	"zombie_nautilus_variant",
];

const USAGE: &str = "usage:
  cargo run -p registry_entries -- generate <data/minecraft dir> <registries.json> <version>";

/// Registry path (without namespace) -> entry name (with namespace) -> numeric ID.
type Ids = BTreeMap<String, HashMap<String, i32>>;

fn main() -> Result<(), Box<dyn Error>> {
	let args: Vec<String> = std::env::args().skip(1).collect();
	let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

	match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
		["generate", data_dir, report, version] => {
			let data_dir = Path::new(data_dir);
			let mut registries = Vec::with_capacity(SYNCED_REGISTRIES.len());
			for registry in SYNCED_REGISTRIES {
				let mut entries = read_names(&data_dir.join(registry))?;
				if entries.is_empty() {
					// The client requires some registries to be non-empty. An empty list most likely
					// means the wrong folder, so stop instead of writing a file that fails at login.
					return Err(format!("registry `{registry}` has no entries").into());
				}
				// The order sets the numeric ID of each entry. The vanilla server sorts by ID.
				entries.sort();
				registries.push((registry, entries));
			}

			let mut ids = builtin_ids(&fs::read_to_string(report)?)?;
			for (registry, entries) in &registries {
				ids.insert(registry.to_string(), entries.iter().enumerate().map(|(i, e)| (format!("minecraft:{e}"), i as i32)).collect());
			}

			let mut tags = Vec::new();
			for registry in ids.keys() {
				let dir = data_dir.join("tags").join(registry);
				if dir.is_dir() {
					tags.push((registry.as_str(), read_tags(&dir)?));
				}
			}
			let tags = resolve_tags(&tags, &ids)?;

			let source = generate(&registries, &tags, version);
			let out = crate_dir.join(OUTPUT_FILE);
			fs::write(&out, source)?;
			println!("entries and tags written to {}", out.display());
		}
		_ => {
			eprintln!("{USAGE}");
			std::process::exit(2);
		}
	}
	Ok(())
}

/// Returns the names of the `.json` files in `dir`, including files in subfolders as
/// `<folder>/<name>`.
fn read_names(dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
	if !dir.is_dir() {
		return Err(format!("{} does not exist. Is the data pack extracted?", dir.display()).into());
	}
	let mut names = Vec::new();
	collect(dir, "", &mut names)?;
	Ok(names)
}

fn collect(dir: &Path, prefix: &str, names: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
	for item in fs::read_dir(dir)? {
		let path = item?.path();
		let name = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| format!("{}: name is not UTF-8", path.display()))?;
		if path.is_dir() {
			collect(&path, &format!("{prefix}{name}/"), names)?;
		} else if let Some(stem) = name.strip_suffix(".json") {
			names.push(format!("{prefix}{stem}"));
		}
	}
	Ok(())
}

/// Reads the built-in registry IDs from the vanilla `registries.json` report.
fn builtin_ids(report_json: &str) -> Result<Ids, Box<dyn Error>> {
	let report: serde_json::Map<String, serde_json::Value> = serde_json::from_str(report_json)?;
	let mut ids = Ids::new();
	for (registry, body) in &report {
		let path = registry.strip_prefix("minecraft:").ok_or_else(|| format!("{registry}: not in the `minecraft` namespace"))?;
		let entries = body["entries"].as_object().ok_or_else(|| format!("{registry}: no `entries` object"))?;
		let mut map = HashMap::with_capacity(entries.len());
		for (entry, value) in entries {
			let id = value["protocol_id"].as_i64().ok_or_else(|| format!("{registry} {entry}: no `protocol_id`"))?;
			map.insert(entry.clone(), id as i32);
		}
		ids.insert(path.to_string(), map);
	}
	Ok(ids)
}

/// Reads every tag file in `dir`. Returns `(tag name, values)`, where each value is an entry name or
/// a `#tag` reference, as written in the file.
fn read_tags(dir: &Path) -> Result<Vec<(String, Vec<String>)>, Box<dyn Error>> {
	let mut tags = Vec::new();
	for name in read_names(dir)? {
		let file = dir.join(format!("{name}.json"));
		let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&file)?)?;
		// The vanilla tags use only plain strings. Other forms ({"id", "required"}) and "replace"
		// would need more handling, so stop instead of writing wrong tags.
		if json.get("replace").is_some() {
			return Err(format!("{}: `replace` is not supported", file.display()).into());
		}
		let values = json["values"].as_array().ok_or_else(|| format!("{}: no `values` array", file.display()))?;
		let values = values
			.iter()
			.map(|v| v.as_str().map(str::to_string).ok_or_else(|| format!("{}: value {v} is not a string", file.display())))
			.collect::<Result<Vec<_>, _>>()?;
		tags.push((format!("minecraft:{name}"), values));
	}
	Ok(tags)
}

/// Resolves `#tag` references and entry names to numeric IDs. Returns
/// `(registry, [(tag, sorted unique IDs)])`, sorted by registry and tag name.
fn resolve_tags(tags: &[(&str, Vec<(String, Vec<String>)>)], ids: &Ids) -> Result<Vec<(String, Vec<(String, Vec<i32>)>)>, Box<dyn Error>> {
	let mut out = Vec::new();
	for (registry, registry_tags) in tags {
		let entry_ids = &ids[*registry];
		let by_name: HashMap<&str, &Vec<String>> = registry_tags.iter().map(|(n, v)| (n.as_str(), v)).collect();

		let mut resolved = Vec::new();
		for (name, _) in registry_tags {
			let mut values = Vec::new();
			expand(name, &by_name, entry_ids, &mut Vec::new(), &mut values).map_err(|e| format!("tag {registry} {name}: {e}"))?;
			values.sort();
			values.dedup();
			resolved.push((name.clone(), values));
		}
		resolved.sort();
		out.push((format!("minecraft:{registry}"), resolved));
	}
	Ok(out)
}

fn expand(tag: &str, tags: &HashMap<&str, &Vec<String>>, entry_ids: &HashMap<String, i32>, stack: &mut Vec<String>, out: &mut Vec<i32>) -> Result<(), String> {
	if stack.iter().any(|t| t == tag) {
		return Err(format!("tag cycle through {tag}"));
	}
	let values = tags.get(tag).ok_or_else(|| format!("unknown tag #{tag}"))?;
	stack.push(tag.to_string());
	for value in values.iter() {
		if let Some(inner) = value.strip_prefix('#') {
			expand(inner, tags, entry_ids, stack, out)?;
		} else {
			out.push(*entry_ids.get(value).ok_or_else(|| format!("unknown entry {value}"))?);
		}
	}
	stack.pop();
	Ok(())
}

/// Turns the entry lists and tags into Rust source.
fn generate(registries: &[(&str, Vec<String>)], tags: &[(String, Vec<(String, Vec<i32>)>)], version: &str) -> String {
	let mut out = format!(
		"//! @generated by `data_collectors/registry_entries` from the vanilla {version} data pack. Do not edit.\n\
		 //!\n\
		 //! The entry IDs of each synchronized registry, in vanilla order, and every vanilla tag that the\n\
		 //! server sends in Update Tags. To write this file again, see\n\
		 //! `data_collectors/registry_entries/README.md`.\n\n\
		 /// The version of the `minecraft:core` known pack that these entries come from.\n\
		 pub const VANILLA_VERSION: &str = \"{version}\";\n\n\
		 /// `(registry ID, entry IDs)` for each synchronized registry. The index of an entry is its numeric ID.\n\
		 pub const SYNCED_REGISTRIES: &[(&str, &[&str])] = &[\n"
	);
	for (registry, entries) in registries {
		out.push_str(&format!("\t(\"minecraft:{registry}\", &[\n"));
		for entry in entries {
			out.push_str(&format!("\t\t\"minecraft:{entry}\",\n"));
		}
		out.push_str("\t]),\n");
	}
	out.push_str(
		"];\n\n\
		 /// `(registry ID, [(tag ID, entry numeric IDs)])` for every vanilla tag of a registry that the\n\
		 /// client knows. Nested tags are already resolved.\n\
		 #[allow(clippy::type_complexity)]\n\
		 pub const VANILLA_TAGS: &[(&str, &[(&str, &[i32])])] = &[\n",
	);
	for (registry, registry_tags) in tags {
		out.push_str(&format!("\t(\"{registry}\", &[\n"));
		for (tag, ids) in registry_tags {
			let ids: Vec<String> = ids.iter().map(i32::to_string).collect();
			out.push_str(&format!("\t\t(\"{tag}\", &[{}]),\n", ids.join(", ")));
		}
		out.push_str("\t]),\n");
	}
	out.push_str("];\n");
	out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(registry: &str, entries: &[(&str, i32)]) -> Ids {
		let mut ids = Ids::new();
		ids.insert(registry.to_string(), entries.iter().map(|(n, i)| (n.to_string(), *i)).collect());
		ids
	}

	fn tag(name: &str, values: &[&str]) -> (String, Vec<String>) {
		(name.to_string(), values.iter().map(|v| v.to_string()).collect())
	}

	#[test]
	fn nested_tags_are_resolved_to_ids() {
		// The client does not resolve "#" references in Update Tags. It needs the final IDs.
		// infiniburn_end = #infiniburn_overworld + bedrock.
		let ids = ids("block", &[("minecraft:netherrack", 5), ("minecraft:bedrock", 2)]);
		let tags = [(
			"block",
			vec![
				tag("minecraft:infiniburn_end", &["#minecraft:infiniburn_overworld", "minecraft:bedrock"]),
				tag("minecraft:infiniburn_overworld", &["minecraft:netherrack"]),
			],
		)];
		let out = resolve_tags(&tags, &ids).unwrap();
		assert_eq!(out[0].0, "minecraft:block");
		assert_eq!(out[0].1[0], ("minecraft:infiniburn_end".to_string(), vec![2, 5]));
	}

	#[test]
	fn unknown_entry_or_tag_is_an_error() {
		// A name that is not in the registry means the IDs come from a different version. Stop,
		// because a wrong ID would put the wrong blocks or items in the tag.
		let ids = ids("block", &[("minecraft:stone", 1)]);
		assert!(resolve_tags(&[("block", vec![tag("minecraft:a", &["minecraft:dirt"])])], &ids).is_err());
		assert!(resolve_tags(&[("block", vec![tag("minecraft:a", &["#minecraft:b"])])], &ids).is_err());
		assert!(resolve_tags(&[("block", vec![tag("minecraft:a", &["#minecraft:a"])])], &ids).is_err());
	}

	#[test]
	fn synced_entries_are_listed_in_the_order_given() {
		// main() sorts before it calls generate(), because the order sets the numeric ID.
		let out = generate(&[("worldgen/biome", vec!["badlands".into(), "plains".into()])], &[], "test");
		assert!(out.find("minecraft:badlands").unwrap() < out.find("minecraft:plains").unwrap(), "{out}");
		assert!(out.contains("(\"minecraft:worldgen/biome\", &["), "{out}");
	}
}
