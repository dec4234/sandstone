use crate::protocol;
use crate::packets::packet_definer::{Packet, PacketState, PacketVersionDefinition};
use serde::{Deserialize, Serialize};

// https://wiki.vg/Protocol
protocol!(v1_20, 764 => {

    // Status-bound
    StatusRequest, StatusRequestBody, 0x00, STATUS => {
        // none
        test: u64
    }
});