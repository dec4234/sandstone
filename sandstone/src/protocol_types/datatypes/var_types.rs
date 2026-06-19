//! The purpose of this file is to define the custom integer types for the Minecraft protocol, VarInt and VarLong.
//! See more details here: https://wiki.vg/Protocol#VarInt_and_VarLong

use crate::network::network_error::NetworkError;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use std::fmt;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;
use tokio::net::TcpStream;
use uuid::Uuid;

const SEGMENT_INT: i32 = 0x7F;
const SEGMENT_LONG: i64 = 0x7F;
const SEGMENT_INT_OPP: i32 = !SEGMENT_INT; // cache these to avoid it at runtime
const SEGMENT_LONG_OPP: i64 = !SEGMENT_LONG;
const CONTINUE_INT: i32 = 0x80;
const CONTINUE_LONG: i64 = 0x80;
pub(crate) const CONTINUE_BYTE: u8 = 0x80; // 10000000

/// A VarInt is a packaged i32. It is represented in a more compressed (on average) byte format than
/// a typical i32. The most significant bit of each byte is used to indicate if there are more bytes
/// to be read, up to a max of 5.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct VarInt(pub i32);

impl VarInt {
	/// Convert a slice of bytes into a VarInt. Reading algorithm taken from https://wiki.vg/
	pub fn from_slice(bytes: &[u8]) -> Result<Self, SerializingErr> {
		if bytes.len() > 5 {
			return Err(SerializingErr::VarTypeTooLong("VarInt must be a max of 5 bytes.".to_string()));
		}

		let mut i: i32 = 0;
		let mut pos = 0;

		for b in bytes {
			let local: i32 = *b as i32;

			i |= (local & SEGMENT_INT) << pos;

			if (local & CONTINUE_INT) == 0 {
				// Terminal byte reached
				return Ok(VarInt(i));
			}

			pos += 7;

			if pos >= 32 {
				return Err(SerializingErr::UniqueFailure("Bit length is too long".to_string()));
			}
		}

		// Ran out of bytes while still expecting a continuation
		Err(SerializingErr::InvalidEndOfVarInt)
	}

	/// Extract a VarInt from a TcpStream. This reads the bytes until it finds a byte that does not have the continue bit set.
	///
	/// This is usually used for reading the packet length VarInt from the start of a packet.
	pub fn from_tcp_stream(stream: &TcpStream) -> Result<Self, NetworkError> {
		let mut buf = [0u8; 3];
		let mut len = 0usize;

		loop {
			let var_buffer = &mut [0u8; 1];
			let n = stream.try_read(var_buffer)?;

			if n == 0 {
				return Err(NetworkError::NoDataReceived);
			}

			let b = var_buffer[0];
			buf[len] = b;
			len += 1;

			if b & crate::network::CONTINUE_BIT == 0 {
				break;
			} else if len >= 3 {
				return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
			}
		}

		Ok(VarInt::from_slice(&buf[..len])?)
	}

	/// Encode this VarInt into a stack-allocated byte array. Returns the array and the number of
	/// bytes written. This avoids heap allocation for serialization hot paths.
	pub fn to_byte_array(&self) -> ([u8; 5], usize) {
		let mut buf = [0u8; 5];
		let mut len = 0;
		let mut inner = self.0;

		loop {
			if (inner & SEGMENT_INT_OPP) == 0 {
				buf[len] = inner as u8;
				len += 1;
				break;
			}

			buf[len] = (inner | CONTINUE_INT) as u8;
			len += 1;

			inner = if inner >= 0 {
				inner >> 7
			} else {
				((inner as u32) >> 7) as i32
			};
		}

		(buf, len)
	}

	/// Convert the VarInt into a Vec of bytes which can be serialized, or converted back to a VarInt using `from_slice`.
	pub fn to_bytes(&self) -> Vec<u8> {
		let (buf, len) = self.to_byte_array();
		buf[..len].to_vec()
	}

	pub fn bytes(i: i32) -> Vec<u8> {
		VarInt(i).to_bytes()
	}
}

impl Display for VarInt {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl FromStr for VarInt {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let bytes = s.as_bytes();

		if bytes.is_empty() || bytes.len() > 5 {
			return Err(Error);
		}

		let var_int = VarInt::from_slice(bytes);

		match var_int {
			Ok(var) => Ok(var),
			Err(_e) => Err(Error),
		}
	}
}

impl McSerialize for VarInt {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let (buf, len) = self.to_byte_array();
		serializer.serialize_bytes(&buf[..len]);
		Ok(())
	}
}

/// Decode a var-type from the deserializer in a single pass, accumulating the value as the bytes
/// are walked and advancing the index past it on success. `max_bytes` is the encoding's byte limit
/// (5 for VarInt, 10 for VarLong). The result is returned as `i64`; callers narrow it to the target
/// width (`as i32` for VarInt). Errors — and leaves the index untouched — if the buffer ends
/// mid-value or the limit is exceeded. Shared by both deserializers so they behave identically.
fn read_var(deserializer: &mut McDeserializer, max_bytes: usize, too_long_msg: &str) -> Result<i64, SerializingErr> {
	let start = deserializer.index;
	let mut value: i64 = 0;
	let mut pos = 0u32;
	let mut offset = 0usize;

	loop {
		if start + offset >= deserializer.data.len() {
			return Err(SerializingErr::InvalidEndOfVarInt);
		}

		let byte = deserializer.data[start + offset];
		offset += 1;

		value |= ((byte & 0x7F) as i64) << pos;

		if byte & CONTINUE_BYTE == 0 {
			deserializer.increment(offset);
			return Ok(value);
		}

		if offset >= max_bytes {
			return Err(SerializingErr::VarTypeTooLong(too_long_msg.to_string()));
		}

		pos += 7;
	}
}

impl McDeserialize for VarInt {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, VarInt> {
		let value = read_var(deserializer, 5, "VarInt must be a max of 5 bytes.")?;

		Ok(VarInt(value as i32))
	}
}

impl From<i32> for VarInt {
	fn from(i: i32) -> Self {
		VarInt(i)
	}
}

impl From<&[u8]> for VarInt {
	/// # Panics
	/// Panics if `bytes` is not a valid VarInt encoding (too long, truncated, or
	/// over-long bit length). Use [`VarInt::from_slice`] to handle malformed input.
	fn from(bytes: &[u8]) -> Self {
		VarInt::from_slice(bytes).unwrap()
	}
}

/// A VarLong is a packaged i64. It is represented in a more compressed (on average) byte format than
/// a typical i64. The most significant bit of each byte is used to indicate if there are more bytes
/// to be read, up to a max of 10.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct VarLong(pub i64);

impl VarLong {
	/// Convert a slice of bytes into a VarLong. Reading algorithm taken from https://wiki.vg/
	pub fn from_slice(bytes: &[u8]) -> SerializingResult<Self> {
		if bytes.len() > 10 {
			return Err(SerializingErr::UniqueFailure("VarLong must be a max of 10 bytes.".to_string()));
		}

		let mut i: i64 = 0;
		let mut pos = 0;

		for b in bytes {
			let local: i64 = *b as i64;

			i |= (local & SEGMENT_LONG) << pos;

			if (local & CONTINUE_LONG) == 0 {
				// Terminal byte reached
				return Ok(VarLong(i));
			}

			pos += 7;

			if pos >= 64 {
				return Err(SerializingErr::UniqueFailure("Bit length is too long".to_string()));
			}
		}

		// Ran out of bytes while still expecting a continuation
		Err(SerializingErr::InvalidEndOfVarInt)
	}

	pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Self, SerializingErr> {
		// cannot use SerializingResult
		VarLong::from_slice(bytes.as_slice())
	}

	/// Encode this VarLong into a stack-allocated byte array. Returns the array and the number of
	/// bytes written.
	pub fn to_byte_array(&self) -> ([u8; 10], usize) {
		let mut buf = [0u8; 10];
		let mut len = 0;
		let mut inner = self.0;

		loop {
			if (inner & SEGMENT_LONG_OPP) == 0 {
				buf[len] = inner as u8;
				len += 1;
				break;
			}

			buf[len] = (inner | CONTINUE_LONG) as u8;
			len += 1;

			inner = if inner >= 0 {
				inner >> 7
			} else {
				((inner as u64) >> 7) as i64
			};
		}

		(buf, len)
	}

	/// Convert the VarLong into a Vec of bytes which can be serialized, or converted back to a VarLong using `from_slice`.
	pub fn to_bytes(&self) -> Vec<u8> {
		let (buf, len) = self.to_byte_array();
		buf[..len].to_vec()
	}

	pub fn bytes(i: i64) -> Vec<u8> {
		VarLong(i).to_bytes()
	}
}

impl Display for VarLong {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl FromStr for VarLong {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let bytes = s.as_bytes();

		if bytes.is_empty() || bytes.len() > 5 {
			return Err(Error);
		}

		let var_int = VarLong::from_slice(bytes);

		match var_int {
			Ok(var) => Ok(var),
			Err(_) => Err(Error),
		}
	}
}

impl McSerialize for VarLong {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let (buf, len) = self.to_byte_array();
		serializer.serialize_bytes(&buf[..len]);
		Ok(())
	}
}

impl McDeserialize for VarLong {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, VarLong> {
		let value = read_var(deserializer, 10, "VarLong must be a max of 10 bytes.")?;

		Ok(VarLong(value))
	}
}

impl From<i64> for VarLong {
	fn from(i: i64) -> Self {
		VarLong(i)
	}
}

impl From<&[u8]> for VarLong {
	/// # Panics
	/// Panics if `bytes` is not a valid VarLong encoding (too long, truncated, or
	/// over-long bit length). Use [`VarLong::from_slice`] to handle malformed input.
	fn from(bytes: &[u8]) -> Self {
		VarLong::from_slice(bytes).unwrap()
	}
}

// For rust stuff go to serializer_types.rs

// 3rd party items

impl McSerialize for Uuid {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.as_u128().mc_serialize(serializer)?; // serialized as u128 in mc protocol

		Ok(())
	}
}

impl McDeserialize for Uuid {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		Ok(Uuid::from_u128(u128::mc_deserialize(deserializer)?))
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};

	#[test]
	fn basic_varint_from_slice() {
		assert_eq!(VarInt::from_slice(&[221, 199, 1]).unwrap(), VarInt(25565));
		assert_eq!(VarInt::from_slice(&[255, 255, 127]).unwrap(), VarInt(2097151));
		assert_eq!(VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap(), VarInt(-1));
		assert_eq!(VarInt::from_slice(&[128, 128, 128, 128, 8]).unwrap(), VarInt(-2147483648));
	}

	#[test]
	fn basic_varint_writing() {
		assert_eq!(VarInt::from_slice(&[221, 199, 1]).unwrap().to_bytes(), vec![221, 199, 1]);
		assert_eq!(VarInt::from_slice(&[255, 255, 127]).unwrap().to_bytes(), vec![255, 255, 127]);
		assert_eq!(VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap().to_bytes(), vec![255, 255, 255, 255, 15]);
	}

	#[test]
	fn basic_varlong_from_slice() {
		assert_eq!(VarLong::from_slice(&[255, 1]).unwrap(), VarLong(255));
		assert_eq!(VarLong::from_slice(&[255, 255, 255, 255, 7]).unwrap(), VarLong(2147483647));
		assert_eq!(VarLong::from_slice(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]).unwrap(), VarLong(-1));
		assert_eq!(VarLong::from_slice(&[128, 128, 128, 128, 248, 255, 255, 255, 255, 1]).unwrap(), VarLong(-2147483648));
	}

	#[test]
	fn basic_varlong_writing() {
		assert_eq!(VarLong::from_slice(&[255, 1]).unwrap().to_bytes(), vec![255, 1]);
		assert_eq!(VarLong::from_slice(&[255, 255, 255, 255, 7]).unwrap().to_bytes(), vec![255, 255, 255, 255, 7]);
		assert_eq!(
			VarLong::from_slice(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]).unwrap().to_bytes(),
			vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1]
		);
		assert_eq!(
			VarLong::from_slice(&[128, 128, 128, 128, 248, 255, 255, 255, 255, 1]).unwrap().to_bytes(),
			vec![128, 128, 128, 128, 248, 255, 255, 255, 255, 1]
		);
	}

	#[test]
	fn test_varint_serialization() {
		let mut serializer = McSerializer::new();

		VarInt(25565).mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(25565, VarInt::mc_deserialize(&mut deserializer).unwrap().0);

		serializer.clear();
		VarInt(2097151).mc_serialize(&mut serializer).unwrap();
		deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(2097151, VarInt::mc_deserialize(&mut deserializer).unwrap().0);

		serializer.clear();
		VarInt(-2147483648).mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.output, vec![128, 128, 128, 128, 8]);

		serializer.clear();
		VarInt(-2147483648).mc_serialize(&mut serializer).unwrap();
		deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(-2147483648, VarInt::mc_deserialize(&mut deserializer).unwrap().0);
	}

	#[test]
	fn test_varlong_serialization() {
		let mut serializer = McSerializer::new();

		VarLong(25565).mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(25565, VarLong::mc_deserialize(&mut deserializer).unwrap().0);

		serializer.clear();
		VarLong(2097151).mc_serialize(&mut serializer).unwrap();
		deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(2097151, VarLong::mc_deserialize(&mut deserializer).unwrap().0);

		serializer.clear();
		VarLong(9223372036854775807).mc_serialize(&mut serializer).unwrap();
		deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(9223372036854775807, VarLong::mc_deserialize(&mut deserializer).unwrap().0);

		serializer.clear();
		VarLong(-2147483648).mc_serialize(&mut serializer).unwrap();
		deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(-2147483648, VarLong::mc_deserialize(&mut deserializer).unwrap().0);

		serializer.clear();
		VarLong(-9223372036854775808).mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.output, vec![128, 128, 128, 128, 128, 128, 128, 128, 128, 1]);
	}

	#[test]
	fn test_zero_handling() {
		assert_eq!(VarInt::from_slice(&[221, 199, 1, 0]).unwrap(), VarInt(25565));
		assert_eq!(VarInt::from_slice(&[255, 255, 127, 0]).unwrap(), VarInt(2097151));
		assert_eq!(VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap(), VarInt(-1));
		assert_eq!(VarInt::from_slice(&[128, 128, 128, 128, 8]).unwrap(), VarInt(-2147483648));
	}

	/// Every value pushed onto the wire must come back identical, otherwise packets
	/// silently corrupt. Covers sign boundaries and the 1-byte/5-byte length edges.
	#[test]
	fn varint_round_trips_over_boundaries() {
		let cases = [0, 1, -1, 127, 128, 25565, 2097151, i32::MAX, i32::MIN];

		for value in cases {
			let mut serializer = McSerializer::new();
			VarInt(value).mc_serialize(&mut serializer).unwrap();
			let mut deserializer = McDeserializer::new(&serializer.output);
			assert_eq!(VarInt::mc_deserialize(&mut deserializer).unwrap().0, value, "round trip failed for {value}");
		}
	}

	#[test]
	fn varlong_round_trips_over_boundaries() {
		let cases = [0, 1, -1, 127, 128, 25565, 2147483647, -2147483648, i64::MAX, i64::MIN];

		for value in cases {
			let mut serializer = McSerializer::new();
			VarLong(value).mc_serialize(&mut serializer).unwrap();
			let mut deserializer = McDeserializer::new(&serializer.output);
			assert_eq!(VarLong::mc_deserialize(&mut deserializer).unwrap().0, value, "round trip failed for {value}");
		}
	}

	/// Deserializing one VarInt must consume exactly its own bytes and leave the rest
	/// intact — packet parsing reads many fields back-to-back from one buffer.
	#[test]
	fn varint_deserialize_advances_index_exactly() {
		let mut serializer = McSerializer::new();
		VarInt(25565).mc_serialize(&mut serializer).unwrap();
		VarInt(42).mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		assert_eq!(VarInt::mc_deserialize(&mut deserializer).unwrap().0, 25565);
		assert_eq!(deserializer.index, 3); // 25565 encodes to 3 bytes
		assert_eq!(VarInt::mc_deserialize(&mut deserializer).unwrap().0, 42);
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn varlong_deserialize_advances_index_exactly() {
		let mut serializer = McSerializer::new();
		VarLong(255).mc_serialize(&mut serializer).unwrap();
		VarLong(1).mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		assert_eq!(VarLong::mc_deserialize(&mut deserializer).unwrap().0, 255);
		assert_eq!(deserializer.index, 2); // 255 encodes to 2 bytes
		assert_eq!(VarLong::mc_deserialize(&mut deserializer).unwrap().0, 1);
		assert!(deserializer.is_at_end());
	}

	/// `from_slice` must reject over-length input rather than reading past the protocol
	/// limit (5 bytes for VarInt, 10 for VarLong).
	#[test]
	fn from_slice_rejects_overlong_input() {
		assert!(VarInt::from_slice(&[128, 128, 128, 128, 128, 1]).is_err());
		assert!(VarLong::from_slice(&[128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 1]).is_err());
	}

	/// A continuation bit on the final available byte means the value was truncated;
	/// deserializing must error instead of returning a half-read value.
	#[test]
	fn deserialize_rejects_truncated_varint() {
		let data = [0x80u8]; // continue bit set, no following byte
		let mut deserializer = McDeserializer::new(&data);
		assert!(VarInt::mc_deserialize(&mut deserializer).is_err());

		let data = [0x80u8];
		let mut deserializer = McDeserializer::new(&data);
		assert!(VarLong::mc_deserialize(&mut deserializer).is_err());
	}

	/// `from_slice` itself (not just the deserializer) must reject a slice whose final
	/// byte still has the continuation bit set — there is no terminal byte.
	#[test]
	fn from_slice_rejects_truncated_input() {
		assert!(VarInt::from_slice(&[0x80]).is_err());
		assert!(VarInt::from_slice(&[255, 255]).is_err());
		assert!(VarLong::from_slice(&[0x80]).is_err());
		assert!(VarLong::from_slice(&[255, 255, 255]).is_err());
	}

	/// Trailing zero padding after a terminal byte must still decode to the same value,
	/// since parsing stops at the terminal byte regardless of what follows.
	#[test]
	fn from_slice_tolerates_trailing_padding() {
		assert_eq!(VarInt::from_slice(&[221, 199, 1, 0]).unwrap(), VarInt(25565));
		assert_eq!(VarLong::from_slice(&[255, 1, 0, 0]).unwrap(), VarLong(255));
	}

	#[test]
	fn deserialize_rejects_empty_input() {
		let data: [u8; 0] = [];
		let mut deserializer = McDeserializer::new(&data);
		assert!(VarInt::mc_deserialize(&mut deserializer).is_err());

		let mut deserializer = McDeserializer::new(&data);
		assert!(VarLong::mc_deserialize(&mut deserializer).is_err());
	}

	/// Display must render the integer value
	#[test]
	fn display_renders_integer_value() {
		assert_eq!(VarInt(25565).to_string(), "25565");
		assert_eq!(VarInt(-1).to_string(), "-1");
		assert_eq!(VarInt(0).to_string(), "0");
		assert_eq!(VarLong(9223372036854775807).to_string(), "9223372036854775807");
		assert_eq!(VarLong(-1).to_string(), "-1");
	}
}
