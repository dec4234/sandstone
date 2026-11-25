//! One region file (`r.<x>.<z>.mca`), which holds the chunks of a 32x32 chunk area.
//!
//! <https://minecraft.wiki/w/Region_file_format>

use crate::protocol::game::world::anvil::chunk::Chunk;
use crate::protocol::game::world::anvil::{AnvilError, AnvilResult, lz4};
use crate::protocol::serialization::{McDeserializer, McSerialize, McSerializer};
use crate::protocol_types::datatypes::nbt::NbtCompound;
use flate2::Compression as Level;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SECTOR: usize = 4096;
/// The location table and the timestamp table, one sector each.
const HEADER_SECTORS: usize = 2;
const MAX_SECTORS: usize = 255;
/// Added to the compression type when the chunk is in a separate `c.<x>.<z>.mcc` file.
const EXTERNAL_FLAG: u8 = 128;

/// The compression of a chunk in a region file. Vanilla writes [Compression::Zlib] unless the
/// server sets `region-file-compression`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
	Gzip = 1,
	Zlib = 2,
	None = 3,
	Lz4 = 4,
}

impl Compression {
	fn from_id(id: u8) -> AnvilResult<Self> {
		match id {
			1 => Ok(Self::Gzip),
			2 => Ok(Self::Zlib),
			3 => Ok(Self::None),
			4 => Ok(Self::Lz4),
			// 127 is a custom algorithm, named in the data. It is for third-party servers only.
			other => Err(AnvilError::UnsupportedCompression(other)),
		}
	}

	fn compress(self, data: &[u8]) -> AnvilResult<Vec<u8>> {
		Ok(match self {
			Self::Gzip => {
				let mut encoder = GzEncoder::new(Vec::new(), Level::default());
				encoder.write_all(data)?;
				encoder.finish()?
			}
			Self::Zlib => {
				let mut encoder = ZlibEncoder::new(Vec::new(), Level::default());
				encoder.write_all(data)?;
				encoder.finish()?
			}
			Self::None => data.to_vec(),
			Self::Lz4 => lz4::compress(data),
		})
	}

	fn decompress(self, data: &[u8]) -> AnvilResult<Vec<u8>> {
		let mut output = Vec::new();
		match self {
			Self::Gzip => {
				GzDecoder::new(data).read_to_end(&mut output)?;
			}
			Self::Zlib => {
				ZlibDecoder::new(data).read_to_end(&mut output)?;
			}
			Self::None => output.extend_from_slice(data),
			Self::Lz4 => output = lz4::decompress(data)?,
		}
		Ok(output)
	}
}

/// The file name of the region that holds chunk `(chunk_x, chunk_z)`, for example `r.0.-1.mca`.
pub fn region_file_name(chunk_x: i32, chunk_z: i32) -> String {
	format!("r.{}.{}.mca", chunk_x >> 5, chunk_z >> 5)
}

/// An open region file. Chunk coordinates are absolute chunk coordinates. Only the low 5 bits
/// select the slot in the file, so the caller must open the region that holds the chunk (see
/// [region_file_name]).
pub struct RegionFile {
	file: File,
	/// The folder of the region file. Chunks larger than 255 sectors are in `c.<x>.<z>.mcc` files here.
	folder: PathBuf,
	/// Per chunk: the first sector in the high 24 bits, the sector count in the low 8 bits. 0 means no chunk.
	locations: [u32; 1024],
	timestamps: [u32; 1024],
	/// One entry per sector of the file. True if a chunk or the header uses it.
	used: Vec<bool>,
}

impl RegionFile {
	/// Open the region file at `path`. If it does not exist, make an empty one.
	pub fn open<P: AsRef<Path>>(path: P) -> AnvilResult<Self> {
		let path = path.as_ref();
		let mut file = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path)?;
		let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();

		let mut header = vec![0u8; HEADER_SECTORS * SECTOR];
		let length = file.metadata()?.len() as usize;
		if length == 0 {
			file.write_all(&header)?;
		} else if length < header.len() {
			return Err(AnvilError::InvalidChunk(format!("{} is {length} bytes, shorter than the region header", path.display())));
		} else {
			file.read_exact(&mut header)?;
		}

		let read_table = |table: &[u8]| -> [u32; 1024] { std::array::from_fn(|i| u32::from_be_bytes(table[i * 4..i * 4 + 4].try_into().expect("4 bytes"))) };
		let locations = read_table(&header[..SECTOR]);
		let timestamps = read_table(&header[SECTOR..]);

		let mut used = vec![false; length.max(header.len()).div_ceil(SECTOR)];
		used[..HEADER_SECTORS].fill(true);
		for &location in &locations {
			let (offset, count) = split_location(location);
			// An entry that points past the end of the file is not marked. Reading it gives an error.
			if let Some(sectors) = used.get_mut(offset..offset + count) {
				sectors.fill(true);
			}
		}

		Ok(Self {
			file,
			folder,
			locations,
			timestamps,
			used,
		})
	}

	/// True if the file has data for the chunk.
	pub fn has_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
		self.locations[index(chunk_x, chunk_z)] != 0
	}

	/// The time of the last write of the chunk, in seconds since the Unix epoch. 0 if it has no data.
	pub fn timestamp(&self, chunk_x: i32, chunk_z: i32) -> u32 {
		self.timestamps[index(chunk_x, chunk_z)]
	}

	/// Read and parse a chunk. Returns `None` if the file has no data for it.
	pub fn read_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> AnvilResult<Option<Chunk>> {
		self.read_chunk_nbt(chunk_x, chunk_z)?.map(Chunk::from_nbt).transpose()
	}

	/// Write a chunk at the coordinates in the chunk.
	pub fn write_chunk(&mut self, chunk: &Chunk, compression: Compression) -> AnvilResult<()> {
		self.write_chunk_nbt(chunk.x, chunk.z, &chunk.to_nbt(), compression)
	}

	/// Read the NBT of a chunk without parsing it. Returns `None` if the file has no data for it.
	pub fn read_chunk_nbt(&mut self, chunk_x: i32, chunk_z: i32) -> AnvilResult<Option<NbtCompound>> {
		let location = self.locations[index(chunk_x, chunk_z)];
		if location == 0 {
			return Ok(None);
		}
		let (offset, count) = split_location(location);
		let invalid = |message: String| AnvilError::InvalidChunk(format!("chunk {chunk_x}, {chunk_z}: {message}"));
		if offset < HEADER_SECTORS || offset + count > self.used.len() {
			return Err(invalid(format!("sectors {offset}..{} are outside the file", offset + count)));
		}

		self.file.seek(SeekFrom::Start((offset * SECTOR) as u64))?;
		let mut header = [0u8; 5];
		self.file.read_exact(&mut header)?;

		// The length counts the compression type byte and the compressed data. Read only those
		// bytes: vanilla does not pad the last chunk of the file to a full sector.
		let length = u32::from_be_bytes(header[..4].try_into().expect("4 bytes")) as usize;
		if length == 0 || length > count * SECTOR - 4 {
			return Err(invalid(format!("length {length} does not fit in {count} sectors")));
		}
		let compression_type = header[4];

		let compressed = if compression_type & EXTERNAL_FLAG != 0 {
			std::fs::read(self.external_path(chunk_x, chunk_z))?
		} else {
			let mut data = vec![0u8; length - 1];
			self.file.read_exact(&mut data)?;
			data
		};

		let bytes = Compression::from_id(compression_type & !EXTERNAL_FLAG)?.decompress(&compressed)?;
		let mut deserializer = McDeserializer::new(&bytes);
		Ok(Some(NbtCompound::from_root(&mut deserializer)?))
	}

	/// Write the NBT of a chunk. The new data goes into free sectors first, then the header points to
	/// it. If the program stops in the middle, the old chunk is still complete.
	pub fn write_chunk_nbt(&mut self, chunk_x: i32, chunk_z: i32, nbt: &NbtCompound, compression: Compression) -> AnvilResult<()> {
		let mut serializer = McSerializer::new();
		// A region chunk is a named root compound. The name is empty in vanilla files.
		if nbt.root_name.is_none() {
			let mut named = nbt.clone();
			named.root_name = Some(String::new());
			named.mc_serialize(&mut serializer)?;
		} else {
			nbt.mc_serialize(&mut serializer)?;
		}
		let compressed = compression.compress(&serializer.output)?;

		let external = 5 + compressed.len() > MAX_SECTORS * SECTOR;
		let mut data = Vec::with_capacity(SECTOR);
		if external {
			std::fs::write(self.external_path(chunk_x, chunk_z), &compressed)?;
			data.extend_from_slice(&1u32.to_be_bytes());
			data.push(compression as u8 | EXTERNAL_FLAG);
		} else {
			data.extend_from_slice(&(compressed.len() as u32 + 1).to_be_bytes());
			data.push(compression as u8);
			data.extend_from_slice(&compressed);
		}
		// The file must stay a multiple of the sector size.
		data.resize(data.len().div_ceil(SECTOR) * SECTOR, 0);
		let count = data.len() / SECTOR;

		let offset = self.find_free(count);
		self.file.seek(SeekFrom::Start((offset * SECTOR) as u64))?;
		self.file.write_all(&data)?;
		if self.used.len() < offset + count {
			self.used.resize(offset + count, false);
		}

		let slot = index(chunk_x, chunk_z);
		let old = self.locations[slot];
		self.locations[slot] = ((offset as u32) << 8) | count as u32;
		self.timestamps[slot] = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as u32).unwrap_or(0);
		self.file.seek(SeekFrom::Start((slot * 4) as u64))?;
		self.file.write_all(&self.locations[slot].to_be_bytes())?;
		self.file.seek(SeekFrom::Start((SECTOR + slot * 4) as u64))?;
		self.file.write_all(&self.timestamps[slot].to_be_bytes())?;

		self.used[offset..offset + count].fill(true);
		let (old_offset, old_count) = split_location(old);
		if old != 0
			&& let Some(sectors) = self.used.get_mut(old_offset..old_offset + old_count)
		{
			sectors.fill(false);
		}

		if !external {
			// A smaller chunk can replace one that was in an external file. Remove the old file.
			match std::fs::remove_file(self.external_path(chunk_x, chunk_z)) {
				Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
				_ => {}
			}
		}

		Ok(())
	}

	/// The first sector of a run of `count` free sectors. It can be past the end of the file.
	fn find_free(&self, count: usize) -> usize {
		let mut run_start = HEADER_SECTORS;
		for (sector, &used) in self.used.iter().enumerate().skip(HEADER_SECTORS) {
			if used {
				run_start = sector + 1;
			} else if sector + 1 - run_start == count {
				return run_start;
			}
		}
		run_start
	}

	fn external_path(&self, chunk_x: i32, chunk_z: i32) -> PathBuf {
		self.folder.join(format!("c.{chunk_x}.{chunk_z}.mcc"))
	}
}

fn index(chunk_x: i32, chunk_z: i32) -> usize {
	((chunk_x & 31) + (chunk_z & 31) * 32) as usize
}

fn split_location(location: u32) -> (usize, usize) {
	((location >> 8) as usize, (location & 0xFF) as usize)
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::protocol_types::datatypes::nbt::NbtByteArray;

	fn temp_dir(name: &str) -> PathBuf {
		let dir = std::env::temp_dir().join(format!("sandstone_region_test_{name}_{}", std::process::id()));
		let _ = std::fs::remove_dir_all(&dir);
		std::fs::create_dir_all(&dir).unwrap();
		dir
	}

	fn nbt(value: i32, bytes: usize) -> NbtCompound {
		let mut compound = NbtCompound::new(Some(""));
		compound.add("value", value);
		// Random bytes do not compress, so the size on disk is about `bytes`.
		let mut x = value as u32 | 1;
		let noise: Vec<i8> = (0..bytes)
			.map(|_| {
				x ^= x << 13;
				x ^= x >> 17;
				x ^= x << 5;
				x as i8
			})
			.collect();
		compound.add("noise", NbtTag::ByteArray(NbtByteArray::new(noise)));
		compound
	}

	use crate::protocol_types::datatypes::nbt::NbtTag;

	#[test]
	fn region_file_names_floor_negative_chunks() {
		assert_eq!(region_file_name(0, 31), "r.0.0.mca");
		assert_eq!(region_file_name(32, -1), "r.1.-1.mca");
		assert_eq!(region_file_name(-33, -32), "r.-2.-1.mca");
	}

	/// Each compression type must write data that it reads back, and the data must still be there
	/// after the file is opened again (the header was written to disk, not only kept in memory).
	#[test]
	fn write_then_read_every_compression() {
		let dir = temp_dir("compression");
		let path = dir.join("r.0.0.mca");
		let types = [Compression::Gzip, Compression::Zlib, Compression::None, Compression::Lz4];
		{
			let mut region = RegionFile::open(&path).unwrap();
			for (i, compression) in types.iter().enumerate() {
				region.write_chunk_nbt(i as i32, 3, &nbt(i as i32, 100), *compression).unwrap();
			}
			assert!(!region.has_chunk(31, 31));
			assert_eq!(region.read_chunk_nbt(31, 31).unwrap(), None);
		}

		let mut region = RegionFile::open(&path).unwrap();
		for i in 0..types.len() as i32 {
			assert!(region.timestamp(i, 3) > 0);
			assert_eq!(region.read_chunk_nbt(i, 3).unwrap(), Some(nbt(i, 100)), "{:?}", types[i as usize]);
		}
		assert_eq!(std::fs::metadata(&path).unwrap().len() % SECTOR as u64, 0, "file must be a multiple of the sector size");
	}

	/// Writing a larger chunk over a smaller one must not overwrite its neighbor, and the freed
	/// sectors must be used again so that the file does not grow forever.
	#[test]
	fn rewrite_moves_and_reuses_sectors() {
		let dir = temp_dir("rewrite");
		let mut region = RegionFile::open(dir.join("r.0.0.mca")).unwrap();

		region.write_chunk_nbt(0, 0, &nbt(1, 100), Compression::None).unwrap(); // sector 2
		region.write_chunk_nbt(1, 0, &nbt(2, 100), Compression::None).unwrap(); // sector 3
		region.write_chunk_nbt(0, 0, &nbt(3, 3 * SECTOR), Compression::None).unwrap(); // grows, moves to 4..8
		assert_eq!(region.read_chunk_nbt(1, 0).unwrap(), Some(nbt(2, 100)));
		assert_eq!(region.read_chunk_nbt(0, 0).unwrap(), Some(nbt(3, 3 * SECTOR)));

		let length = std::fs::metadata(dir.join("r.0.0.mca")).unwrap().len();
		region.write_chunk_nbt(2, 0, &nbt(4, 100), Compression::None).unwrap();
		assert_eq!(split_location(region.locations[2]).0, 2, "sector 2 was freed and must be used again");
		assert_eq!(std::fs::metadata(dir.join("r.0.0.mca")).unwrap().len(), length);
	}

	/// A chunk larger than 255 sectors goes into a `.mcc` file. If it gets smaller, the `.mcc` file
	/// must go away, or a later read could use stale data.
	#[test]
	fn large_chunks_use_an_external_file() {
		let dir = temp_dir("external");
		let mut region = RegionFile::open(dir.join("r.-1.0.mca")).unwrap();
		let big = nbt(5, (MAX_SECTORS + 1) * SECTOR);

		region.write_chunk_nbt(-1, 2, &big, Compression::Zlib).unwrap();
		assert!(dir.join("c.-1.2.mcc").exists());
		assert_eq!(split_location(region.locations[index(-1, 2)]).1, 1);
		assert_eq!(region.read_chunk_nbt(-1, 2).unwrap(), Some(big));

		region.write_chunk_nbt(-1, 2, &nbt(6, 10), Compression::Zlib).unwrap();
		assert!(!dir.join("c.-1.2.mcc").exists());
		assert_eq!(region.read_chunk_nbt(-1, 2).unwrap(), Some(nbt(6, 10)));
	}

	/// Vanilla 26.3 does not pad the last chunk of a file to a full sector. A reader that reads
	/// whole sectors fails on every vanilla region file.
	#[test]
	fn reads_an_unpadded_last_chunk() {
		let dir = temp_dir("unpadded");
		let path = dir.join("r.0.0.mca");
		RegionFile::open(&path).unwrap().write_chunk_nbt(0, 0, &nbt(1, 100), Compression::Zlib).unwrap();

		let mut bytes = std::fs::read(&path).unwrap();
		let length = u32::from_be_bytes(bytes[2 * SECTOR..2 * SECTOR + 4].try_into().unwrap()) as usize;
		bytes.truncate(2 * SECTOR + 4 + length);
		std::fs::write(&path, bytes).unwrap();

		let mut region = RegionFile::open(&path).unwrap();
		assert_eq!(region.read_chunk_nbt(0, 0).unwrap(), Some(nbt(1, 100)));
		// A new chunk goes after it, not over it.
		region.write_chunk_nbt(1, 0, &nbt(2, 100), Compression::Zlib).unwrap();
		assert_eq!(region.read_chunk_nbt(0, 0).unwrap(), Some(nbt(1, 100)));
		assert_eq!(region.read_chunk_nbt(1, 0).unwrap(), Some(nbt(2, 100)));
	}

	#[test]
	fn unknown_compression_is_an_error() {
		let dir = temp_dir("unknown");
		let path = dir.join("r.0.0.mca");
		RegionFile::open(&path).unwrap().write_chunk_nbt(0, 0, &nbt(1, 10), Compression::None).unwrap();

		// Change the compression type byte of the chunk at sector 2 to 127 (custom).
		let mut bytes = std::fs::read(&path).unwrap();
		bytes[2 * SECTOR + 4] = 127;
		std::fs::write(&path, bytes).unwrap();

		let result = RegionFile::open(&path).unwrap().read_chunk_nbt(0, 0);
		assert!(matches!(result, Err(AnvilError::UnsupportedCompression(127))), "{result:?}");
	}
}
