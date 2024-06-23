//! # Register-RS Example Application

use register_rs::*;

#[derive(Register, ReadableRegister, WriteableRegister, Debug)]
#[register(address = 0xC0, length = 1, endian = "big")]
pub struct MyReadableRegister {
    /// Reserved
    #[register(reset = 0b11111, bits = "0..4")]
    _reserved: u8,
    /// A Boolean register
    #[register(bit = "5", reset = false)]
    boolean_state: bool,
    // #[register(bit = "6", reset = true)]
    // boolean_state_2: bool,
    /// A non-specified option
    #[register(bits = "6..7", reset = SomeState::StateOne)]
    something_else: SomeState
}

#[derive(Clone, Copy, Debug, TryValued)]
// #[valued(default = Self::Invalid)]
enum SomeState {
    #[valued(0)]
    StateOne,
    #[valued(1)]
    StateTwo,
    #[valued(None)]
    Invalid
}

fn main() {
    let mut my_register = MyReadableRegister::reset_value();
    // my_register.something_else = SomeState::StateTwo;

    // Base impl
    println!("ADDRESS = {:?}\nLENGTH = {:?}", MyReadableRegister::ADDRESS, MyReadableRegister::LENGTH);
    println!("MyReadableRegister: {:?}", my_register);

    // Writeable impl
    let buffer = my_register.into_bytes().expect("Something went horribly wrong");
    println!("Buffer[{}]: {:?}\nBuffer[0]: {:b}", buffer.len(), buffer, buffer[0]);


    // Readable impl
    let read_register = MyReadableRegister::from_bytes(&buffer);
    println!("Read: {:?}", read_register);
}