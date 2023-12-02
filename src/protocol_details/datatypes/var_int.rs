pub struct VarInt;

impl Trait for VarInt {

}

pub trait Trait {
    fn hello() {
        println!("Hello");
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol_details::datatypes::var_int::{Trait, VarInt};

    #[test]
    fn basic() {
        let var = VarInt::hello();
    }
}