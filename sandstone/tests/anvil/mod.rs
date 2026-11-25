//! Tests of the Anvil module with region data that a vanilla 26.3 server wrote.
//!
//! `assets/r.0.-1.mca` holds two chunks copied byte for byte from vanilla region files: chunk (6, -2)
//! with zlib compression and chunk (7, -2) with LZ4 compression (`region-file-compression=lz4`).

use sandstone::protocol::game::world::anvil::{BlockState, Chunk, Compression, DATA_VERSION, Dimension, RegionFile};
use sandstone::protocol::game::world::block_states::BEDROCK;
use sandstone::protocol::packets::ChunkDataUpdateLightPacket;
use sandstone::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use std::path::{Path, PathBuf};

const FIXTURE: &[u8] = include_bytes!("assets/r.0.-1.mca");

fn temp_dir(name: &str) -> PathBuf {
	let dir = std::env::temp_dir().join(format!("sandstone_anvil_{name}_{}", std::process::id()));
	let _ = std::fs::remove_dir_all(&dir);
	std::fs::create_dir_all(&dir).unwrap();
	dir
}

/// The checks that every full vanilla chunk must pass:
/// - read then write gives the same NBT (no data is lost when a server saves a chunk it loaded)
/// - the packet goes over the wire and comes back the same (the client can read it)
/// - a chunk made from the packet makes the same packet again (a server can save what it sends)
fn check_chunk(region: &mut RegionFile, x: i32, z: i32) -> Option<ChunkDataUpdateLightPacket> {
	let nbt = region.read_chunk_nbt(x, z).unwrap()?;
	let chunk = Chunk::from_nbt(nbt.clone()).unwrap_or_else(|e| panic!("chunk {x}, {z}: {e}"));
	assert_eq!(chunk.to_nbt(), nbt, "chunk {x}, {z}: NBT changed in a read and write");
	assert_eq!((chunk.x, chunk.z), (x, z));

	if chunk.status != "minecraft:full" {
		return None;
	}

	let dimension = Dimension::overworld();
	let packet = chunk.to_packet(&dimension).unwrap_or_else(|e| panic!("chunk {x}, {z}: {e}"));

	let mut serializer = McSerializer::new();
	packet.mc_serialize(&mut serializer).unwrap();
	let bytes: Vec<u8> = serializer.into();
	let mut deserializer = McDeserializer::new(&bytes);
	assert_eq!(ChunkDataUpdateLightPacket::mc_deserialize(&mut deserializer).unwrap(), packet, "chunk {x}, {z}: wire round trip");
	assert!(deserializer.is_at_end());

	let back = Chunk::from_packet(&packet, &dimension).unwrap();
	assert_eq!(back.to_packet(&dimension).unwrap(), packet, "chunk {x}, {z}: packet -> chunk -> packet");

	Some(packet)
}

#[test]
fn vanilla_zlib_and_lz4_chunks() {
	let dir = temp_dir("fixture");
	let path = dir.join("r.0.-1.mca");
	std::fs::write(&path, FIXTURE).unwrap();
	let mut region = RegionFile::open(&path).unwrap();

	for (x, z) in [(6, -2), (7, -2)] {
		let chunk = region.read_chunk(x, z).unwrap().unwrap();
		assert_eq!(chunk.data_version, DATA_VERSION);
		assert_eq!(chunk.status, "minecraft:full");

		let packet = check_chunk(&mut region, x, z).unwrap();
		// The world floor (y = -64) is all bedrock. If the section order or the YZX index order were
		// wrong, other blocks would be here.
		let bottom = packet.data.data.data[0].block_states.values(4096).unwrap();
		assert!(bottom[..256].iter().all(|&id| id == BEDROCK), "chunk {x}, {z}: floor is not bedrock");
		assert!(packet.data.data.data.iter().all(|s| s.block_count >= 0 && s.block_count <= 4096));
	}

	// The zlib chunk has a sculk sensor, so the block entity type must be its protocol ID.
	let chunk = region.read_chunk(6, -2).unwrap().unwrap();
	let sensor = chunk
		.block_entities
		.iter()
		.position(|e| e.get_string("id").as_deref() == Some("minecraft:sculk_sensor"))
		.expect("vanilla chunk has a sculk sensor");
	let packet = chunk.to_packet(&Dimension::overworld()).unwrap();
	let entity = &packet.data.block_entities.slice()[sensor];
	let block = chunk.block_entities[sensor].clone();
	let (bx, by, bz) = (block.get("x").cloned(), block.get("y").cloned(), block.get("z").cloned());
	assert_eq!(
		format!("{bx:?} {by:?} {bz:?}"),
		format!(
			"Some(Int({})) Some(Int({})) Some(Int({}))",
			6 * 16 + entity.packed_xz.x() as i32,
			entity.y,
			-2 * 16 + entity.packed_xz.z() as i32
		)
	);

	// The block at the sensor's position must be a sculk sensor in the packet too.
	let section = &packet.data.data.data[((entity.y as i32 >> 4) + 4) as usize];
	let index = ((entity.y as usize & 15) << 8) | ((entity.packed_xz.z() as usize) << 4) | entity.packed_xz.x() as usize;
	let state = BlockState::from_id(section.block_states.values(4096).unwrap()[index]).unwrap();
	assert_eq!(state.name, "minecraft:sculk_sensor");
}

/// A chunk written with any compression must read back the same, in a new file and in a file
/// that vanilla wrote.
#[test]
fn rewrite_vanilla_chunks() {
	let dir = temp_dir("rewrite");
	std::fs::write(dir.join("r.0.-1.mca"), FIXTURE).unwrap();
	let mut source = RegionFile::open(dir.join("r.0.-1.mca")).unwrap();
	let chunks = [source.read_chunk(6, -2).unwrap().unwrap(), source.read_chunk(7, -2).unwrap().unwrap()];

	for compression in [Compression::Gzip, Compression::Zlib, Compression::None, Compression::Lz4] {
		let path = dir.join(format!("{compression:?}.mca"));
		let mut region = RegionFile::open(&path).unwrap();
		for chunk in &chunks {
			region.write_chunk(chunk, compression).unwrap();
		}
		drop(region);

		let mut region = RegionFile::open(&path).unwrap();
		for chunk in &chunks {
			assert_eq!(region.read_chunk(chunk.x, chunk.z).unwrap().as_ref(), Some(chunk), "{compression:?}");
		}
	}

	// Write over the vanilla chunks in the vanilla file, swapped, so both must move.
	source.write_chunk(&chunks[1], Compression::Lz4).unwrap();
	source.write_chunk(&chunks[0], Compression::Zlib).unwrap();
	let mut source = RegionFile::open(dir.join("r.0.-1.mca")).unwrap();
	assert_eq!(source.read_chunk(6, -2).unwrap().as_ref(), Some(&chunks[0]));
	assert_eq!(source.read_chunk(7, -2).unwrap().as_ref(), Some(&chunks[1]));
}

/// Read every chunk of every region file in the folders in `SANDSTONE_VANILLA_REGIONS` (separated
/// by `;`). Make the folders with a vanilla server, then run:
///
/// `SANDSTONE_VANILLA_REGIONS=<world>/dimensions/minecraft/overworld/region cargo test -p sandstone --test mod -- --ignored`
#[test]
#[ignore = "needs vanilla region files, see the doc comment"]
fn every_chunk_of_vanilla_worlds() {
	let folders = std::env::var("SANDSTONE_VANILLA_REGIONS").expect("set SANDSTONE_VANILLA_REGIONS to one or more region folders");
	let (mut chunks, mut full) = (0, 0);

	for folder in folders.split(';') {
		let mut paths: Vec<PathBuf> = std::fs::read_dir(folder)
			.unwrap()
			.map(|e| e.unwrap().path())
			.filter(|p| p.extension().is_some_and(|e| e == "mca"))
			.collect();
		paths.sort();
		assert!(!paths.is_empty(), "{folder} has no region files");

		for path in paths {
			let (rx, rz) = region_coords(&path);
			let mut region = RegionFile::open(&path).unwrap();
			for z in 0..32 {
				for x in 0..32 {
					let (cx, cz) = (rx * 32 + x, rz * 32 + z);
					if !region.has_chunk(cx, cz) {
						continue;
					}
					chunks += 1;
					full += check_chunk(&mut region, cx, cz).is_some() as usize;
				}
			}
		}
	}

	println!("checked {chunks} chunks, {full} full chunks went through the packet checks");
	assert!(full > 0, "no full chunks were checked");
}

fn region_coords(path: &Path) -> (i32, i32) {
	let name = path.file_name().unwrap().to_str().unwrap();
	let parts: Vec<&str> = name.split('.').collect();
	(parts[1].parse().unwrap(), parts[2].parse().unwrap())
}
