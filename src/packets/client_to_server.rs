use crate::define_packet;
use serde::{Deserialize, Serialize};

define_packet!(PingRequestStruct, PingRequest, SERVER => {
    PingRequestUniversal, 0x01, 0, 16000 => {
        payload, u64
    }
});

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {

    }
}
