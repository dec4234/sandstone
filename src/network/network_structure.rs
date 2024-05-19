use anyhow::Result;

use crate::network::connection::CraftConnection;

pub trait LoginHandler {
	fn handle_login(&mut self, connection: &mut CraftConnection) -> Result<()>;
}