//! Protocol player state information

use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::util::java::bitset::BitSet;
use sandstone_derive::{McDeserialize, McSerialize};

#[derive(McSerialize, McDeserialize, Debug, Clone, Hash, PartialEq)]
pub struct PackedPlayerInfoUpdate {
    data: BitSet,
}

impl PackedPlayerInfoUpdate {
    pub fn new() -> Self {
        Self {
            data: BitSet::new(8),
        }
    }

    pub fn set(&mut self, update_type: PlayerInfoUpdateType, set: bool) -> SerializingResult<()> {
        self.data.set_bit(update_type.get_mask() as usize, set);
        Ok(())
    }

    pub fn get(&self, update_type: PlayerInfoUpdateType) -> bool {
        self.data.get_bit(update_type.get_mask() as usize)
    }
}

#[derive(McSerialize, Debug, Clone, Hash, PartialEq)]
pub enum PlayerInfoUpdateType {
    AddPlayer,
    InitializeChat,
    UpdateGameMode,
    UpdateListed,
    UpdateLatency,
    UpdateDisplayName,
    UpdateListPriority,
    UpdateHat,
}

impl PlayerInfoUpdateType {
    pub fn get_mask(&self) -> u8 {
        match self {
            PlayerInfoUpdateType::AddPlayer => 0x01,
            PlayerInfoUpdateType::InitializeChat => 0x02,
            PlayerInfoUpdateType::UpdateGameMode => 0x04,
            PlayerInfoUpdateType::UpdateListed => 0x08,
            PlayerInfoUpdateType::UpdateLatency => 0x10,
            PlayerInfoUpdateType::UpdateDisplayName => 0x20,
            PlayerInfoUpdateType::UpdateListPriority => 0x40,
            PlayerInfoUpdateType::UpdateHat => 0x80,
        }
    }
}

// todo https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Info_Update
/*#[derive(McSerialize, McDeserialize, Debug, Clone, Hash, PartialEq)]
pub enum PlayerInfoAction {
    AddPlayer((String, String))
}*/
