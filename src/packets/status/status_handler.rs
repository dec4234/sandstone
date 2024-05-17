use base64::alphabet;
use base64::alphabet::Alphabet;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};

const ALPHABET: Alphabet = alphabet::STANDARD;
const CONFIG: GeneralPurposeConfig = GeneralPurposeConfig::new();
const ENGINE: GeneralPurpose = GeneralPurpose::new(&ALPHABET, CONFIG);

