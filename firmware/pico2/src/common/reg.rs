#[repr(usize)]
#[derive(Clone, Copy)]
pub enum RegAddr {
    RESET = 0x4002_0000,
    IO_BANK0 = 0x4002_8000,
    SIO = 0xd000_0000,
    PADS_BANK0 = 0x4003_8000,
}
