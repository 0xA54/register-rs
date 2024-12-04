#[derive(Register, ReadableRegister, WriteableRegister)]
#[register(address = 0x31, length = 1, namespace = Spirit1Standard, endian = "little")]
pub struct PcktCtrl3 {
    /// Format of packet (*see Section 9.7*)
    #[register(bits = "6..7", reset = PacketFormat::Basic)]
    pub pckt_frmt: PacketFormat,
    /// `RX_MODE`
    #[register(bits = "4..5", reset = RxMode::Normal)]
    pub rx_mode: RxMode,
    /// Size in number of binary digit of length field 
    #[register(bits = "0..3", reset = 0b0111)]
    pub len_wid: u8,
}

// RegisterEnum can never fail if it must implement an invalid translation
// In the implementation impl Into<Result<T, E>> for Enum...
// Automatically figure out width and whether default value is required
#[derive(RegisterEnum)]
#[register(type = u8)]
pub enum PacketFormat {
    #[value(0)]
    Basic,
    #[value(1)]
    WMBus,
    #[value(2)]
    STack,
    #[default]
    Invalid,
}

#[derive(Register, ReadableRegister)]
#[register(address = 0xFF, length = 1, endian = "little", namespace = Spirit1Extended)]
pub struct SimpleRegister(u8);
