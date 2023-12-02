use crate::define_packet;

define_packet!(PingRequestStruct, PingRequest => {
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
