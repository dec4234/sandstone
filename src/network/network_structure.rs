use anyhow::Result;

use crate::network::connection::CraftClient;

pub trait LoginHandler {
	fn handle_login(&mut self, connection: &mut CraftClient) -> Result<()>;
}