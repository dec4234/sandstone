use crate::protocol;
use crate::packets::packet_definer::{Packet, PacketState, PacketVersionDefinition};
use serde::{Deserialize, Serialize};

// https://wiki.vg/Protocol
protocol!(v1_20, 764 => {

    // Status-bound
    StatusRequest, StatusRequestBody, 0x00, STATUS => {
        // none
    }
});

#[cfg(test)]
mod tests {
    use nbt::Value::{Compound};
    use crate::map;
    use crate::packets::versions::v1_20::{StatusRequestBody, v1_20};

    #[test]
    fn basic() {
        let p = v1_20::StatusRequest(StatusRequestBody {

        });
    }
}