use anyhow::Result;

pub struct CraftConnection<T> {
	connection: T
}

impl<T: CraftConnectionExt> CraftConnection<T> {
	pub fn from_connection(connection: T) -> Self {
		Self {
			connection
		}
	}
}

/**
TODO: add new trait for packaged packet that accepts packet id to short circuit deserialization?
*/

pub trait CraftConnectionExt {
	fn read_packet(&mut self) -> Result<Vec<u8>>;
}

impl CraftConnectionExt for tokio::net::TcpStream {
	fn read_packet(&mut self) -> Result<Vec<u8>> {
		todo!()
	}
}

impl CraftConnectionExt for std::net::TcpStream {
	fn read_packet(&mut self) -> Result<Vec<u8>> {
		todo!()
	}
}