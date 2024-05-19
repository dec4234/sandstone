use anyhow::Result;
use base64::alphabet;
use base64::alphabet::Alphabet;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use log::debug;

use crate::network::connection::CraftConnection;

const ALPHABET: Alphabet = alphabet::STANDARD;
const CONFIG: GeneralPurposeConfig = GeneralPurposeConfig::new();
const ENGINE: GeneralPurpose = GeneralPurpose::new(&ALPHABET, CONFIG);

pub fn handle_status(connection: &mut CraftConnection) -> Result<()> {
	debug!("Handling status for {}", connection);
	
	// TODO: make status packet, send it, and handle the response
	
	Ok(())
}