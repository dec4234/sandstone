//! Get block state IDs from the vanilla server and write them as Rust constants.
//!
//! Two steps:
//! - `report <version>` downloads the vanilla server jar for `<version>` from Mojang and runs its data
//!   generator. The generator writes `generated/reports/blocks.json` into `work/<version>/`.
//! - `generate <blocks.json> <version>` reads that report and writes
//!   `sandstone/src/protocol/game/world/block_states.rs`.
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

	#[test]
	fn names_that_collide_are_an_error() {
		let report = r#"{
			"minecraft:a_b": { "states": [ { "id": 0, "default": true } ] },
			"minecraft:a.b": { "states": [ { "id": 1, "default": true } ] }
		}"#;
		assert!(generate(report, "test").is_err());
	}
}
