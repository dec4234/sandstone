use base64::{alphabet, Engine, engine};
use base64::alphabet::Alphabet;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use serde::{Deserialize, Serialize};
use crate::packets::mc_serializer::McSerializer;

const ALPHABET: Alphabet = alphabet::STANDARD;
const CONFIG: GeneralPurposeConfig = GeneralPurposeConfig::new();
const ENGINE: GeneralPurpose = GeneralPurpose::new(&ALPHABET, CONFIG);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSpec {
    version: StatusResponseVersionInfo,
    players: StatusResponsePlayersInfo,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    favicon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enforcesSecureChat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previewsChat: Option<bool>,
}

impl StatusSpec {
    pub fn encode_image(&mut self, bytes: Vec<u8>) {
        let based = ENGINE.encode(bytes);
        // TODO: add favicon prefix
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusResponseVersionInfo {
    name: String,
    protocol: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusResponsePlayersInfo {
    max: u32,
    online: u32,
    sample: Vec<StatusResponseUserInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusResponseUserInfo {
    name: String,
    id: String, // UUID
}

#[test]
fn serialization_test() {
    let spec = StatusSpec {
        version: StatusResponseVersionInfo {
            name: "1.19.4".to_string(),
            protocol: 758,
        },
        players: StatusResponsePlayersInfo {
            max: 10,
            online: 0,
            sample: vec![],
        },
        description: "".to_string(),
        favicon: None,
        enforcesSecureChat: None,
        previewsChat: None,
    };

    let out = serde_json::to_string(&spec).unwrap();
    println!("{}", out);

    /*let mut serializer = McSerializer::new();
    spec.serialize(&mut serializer).unwrap();
    println!("{:?}", serializer.output);*/
}