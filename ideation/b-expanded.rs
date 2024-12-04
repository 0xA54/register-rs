
#[doc = "..."] // <<< Make a function to make a table of key values
pub struct PcktCtrl3 {
    pub pckt_frmt: PacketFormat,
    pub rx_mode: RxMode,
    pub len_wid: u8,
}

pub struct __PcktCtrl3_repr();

impl __PcktCtrl3_repr {
    const LENGTH: usize = 1;
    const ADDRESS: u8 = 0x31;

    const RESET_MASK: u8 = 0b0000_0111;

    #[inline(always)]
    #[doc = "Format of packet (*see Section 9.7*)"]
    #[doc = "..."] // <<< Make a function to make a table of key values
    const fn set_pckt_frmt(base: &mut [u8; Self::LENGTH], value: PacketFormat) {
        let value_into_word: u8 = const { 
            // do static_assert on length
            let t = value.into();
            t
        };
    }

    #[inline(always)]
    const fn get_pckt_frmt(base: &[u8; Self::LENGTH]) -> RegisterResult<PacketFormat> {

    }

    ...

}

impl Spirit1Standard for PcktCtrl3 {}

impl Register<u8> for PcktCtrl3 {
    const ADDRESS: WORD = 0x31;
    const LENGTH: usize = 1;

    const fn reset_value() -> Self {
        Self {
            pckt_frmt: PacketFormat::Basic,
            rx_mode: RxMode::Normal,
            len_wid: 0b0111,
        }
    }
}

impl ReadableRegister<u8> for PcktCtrl3 {
    fn from_bytes(buffer: &[u8; Self::LENGTH]) -> RegisterResult<Self> {
        Ok(Self {
            pckt_frmt: Self::get_pckt_frmt(&buffer)?,
            rx_mode: Self::get_rx_mode(&buffer)?,
            len_wid: Self::get_len_wid(&buffer)?
        })
    }
}

impl WriteableRegister for PcktCtrl3 {
    fn into_bytes(&self) -> RegisterResult<[u8; Self::LENGTH]> {
        // let mut base: [u8; Self::LENGTH] = 
    }
}


// ////////////////////////

fn main() {
    let mut radio = Radio();

    let (pckt_frmt, rx_mode) = read!{
        radio,
        PcktCtrl3::pckt_frmt,
        PcktCtrl3::rx_mode,
    };
    // == (without const expansion and namespace grouping)
    let (pckt_frmt, rx_mode) = {
        // do this by each namespace...

        const START_ADDRESS: u8 = {
            // order and get start address
        };
        const LENGTH: usize = 1;

        let raw: [u8; LENGTH] = radio.read_raw::<Spirit1Standard>(START_ADDRESS, LENGTH)?;

        let pckt_frmt = __PcktCtrl3_repr_::get_pckt_frmt(&raw)?;
        let rx_mode = __PcktCtrl3_repr_::get_rx_mode(&raw)?;

        (pckt_frmt, rx_mode)
    };
}