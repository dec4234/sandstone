use anyhow::Result;

use crate::network::connection::CraftClient;

pub trait LoginHandler {
	fn handle_login(connection: &mut CraftClient) -> Result<()>;
}