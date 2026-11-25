//! The LZ4 stream format of region files (compression type 4).
//!
//! Vanilla uses `LZ4BlockOutputStream` from lz4-java. This is not the standard LZ4 frame format. The
//! stream is a list of blocks, and an empty block ends it. Each block has a 21 byte header:
//! - the magic `LZ4Block` (8 bytes)
//! - a token: the method (0x10 raw, 0x20 LZ4) or'd with the level (the block size is `1 << (10 + level)`)
//! - the compressed length, the decompressed length, and a checksum (each a little-endian i32)
//!
//! The checksum is the xxHash32 of the decompressed bytes with seed `0x9747b28c`, masked to 28 bits.

use crate::protocol::game::world::anvil::{AnvilError, AnvilResult};
use xxhash_rust::xxh32::xxh32;

const MAGIC: &[u8; 8] = b"LZ4Block";
const HEADER_LENGTH: usize = MAGIC.len() + 1 + 4 + 4 + 4;
const METHOD_RAW: u8 = 0x10;
const METHOD_LZ4: u8 = 0x20;
const SEED: u32 = 0x9747b28c;
/// The lz4-java default block size is 64 KiB, which is level 6.
const LEVEL: u8 = 6;
const BLOCK_SIZE: usize = 1 << (10 + LEVEL);

fn checksum(data: &[u8]) -> u32 {
	xxh32(data, SEED) & 0x0FFF_FFFF
}

fn invalid(message: String) -> AnvilError {
	AnvilError::InvalidChunk(format!("LZ4: {message}"))
}

pub(crate) fn decompress(mut input: &[u8]) -> AnvilResult<Vec<u8>> {
	let mut output = Vec::new();

	loop {
		if input.len() < HEADER_LENGTH || &input[..MAGIC.len()] != MAGIC {
			return Err(invalid("missing block header".to_string()));
		}
		let token = input[8];
		let read_len = |at: usize| i32::from_le_bytes(input[at..at + 4].try_into().expect("4 bytes")) as usize;
		let (compressed, decompressed, check) = (read_len(9), read_len(13), read_len(17) as u32);
		let level = token & 0x0F;
		if decompressed > 1 << (10 + level) {
			return Err(invalid(format!("block of {decompressed} bytes is larger than level {level} allows")));
		}

		if decompressed == 0 {
			// The end block. lz4-java writes it with lengths and checksum 0.
			return Ok(output);
		}

		let body = input.get(HEADER_LENGTH..HEADER_LENGTH + compressed).ok_or_else(|| invalid("input ended in a block".to_string()))?;
		let start = output.len();
		match token & 0xF0 {
			METHOD_RAW if compressed == decompressed => output.extend_from_slice(body),
			METHOD_LZ4 => {
				output.resize(start + decompressed, 0);
				let written = lz4_flex::block::decompress_into(body, &mut output[start..]).map_err(|e| invalid(e.to_string()))?;
				if written != decompressed {
					return Err(invalid(format!("block gave {written} bytes, header says {decompressed}")));
				}
			}
			_ => return Err(invalid(format!("unknown block token {token:#x}"))),
		}

		if checksum(&output[start..]) != check {
			return Err(invalid("checksum does not match".to_string()));
		}
		input = &input[HEADER_LENGTH + compressed..];
	}
}

pub(crate) fn compress(input: &[u8]) -> Vec<u8> {
	let mut output = Vec::with_capacity(input.len() / 2);

	let mut write_block = |method: u8, body: &[u8], decompressed: usize, check: u32| {
		output.extend_from_slice(MAGIC);
		output.push(method | LEVEL);
		output.extend_from_slice(&(body.len() as i32).to_le_bytes());
		output.extend_from_slice(&(decompressed as i32).to_le_bytes());
		output.extend_from_slice(&check.to_le_bytes());
		output.extend_from_slice(body);
	};

	for block in input.chunks(BLOCK_SIZE) {
		let compressed = lz4_flex::block::compress(block);
		// Like lz4-java, keep the raw bytes when compression does not make the block smaller.
		if compressed.len() < block.len() {
			write_block(METHOD_LZ4, &compressed, block.len(), checksum(block));
		} else {
			write_block(METHOD_RAW, block, block.len(), checksum(block));
		}
	}
	write_block(METHOD_RAW, &[], 0, 0);

	output
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn round_trip_over_many_blocks() {
		// Several blocks, one that compresses and one (random-ish) that stays raw.
		let mut data: Vec<u8> = (0..BLOCK_SIZE * 2).map(|i| (i % 7) as u8).collect();
		let mut x = 1u32;
		data.extend((0..BLOCK_SIZE / 2).map(|_| {
			x = x.wrapping_mul(1103515245).wrapping_add(12345);
			(x >> 16) as u8
		}));

		let compressed = compress(&data);
		assert_eq!(decompress(&compressed).unwrap(), data);
		assert_eq!(decompress(&compress(&[])).unwrap(), Vec::<u8>::new());
	}

	/// A wrong checksum means the data is damaged. It must be an error, not a chunk with bad blocks.
	#[test]
	fn bad_checksum_is_an_error() {
		let mut compressed = compress(b"hello hello hello hello hello");
		compressed[17] ^= 1;
		assert!(decompress(&compressed).is_err());
	}

	/// The layout that `LZ4BlockOutputStream` writes for a short input: one raw block with token 0x16
	/// (raw, level 6), then the end block. This test only checks the layout. The check against real
	/// vanilla output is the test that reads LZ4 region files from a vanilla server.
	#[test]
	fn reads_the_lz4_java_layout() {
		let check = checksum(b"hello");
		let mut stream = Vec::new();
		stream.extend_from_slice(b"LZ4Block\x16\x05\0\0\0\x05\0\0\0");
		stream.extend_from_slice(&check.to_le_bytes());
		stream.extend_from_slice(b"hello");
		stream.extend_from_slice(b"LZ4Block\x16\0\0\0\0\0\0\0\0\0\0\0\0");
		assert_eq!(decompress(&stream).unwrap(), b"hello");
		assert_eq!(compress(b"hello"), stream);
	}
}
