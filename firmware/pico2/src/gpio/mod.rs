pub mod gpio;

#[repr(C)]
struct GpioRegs{
    pub status: u32,  // Status Register
    pub ctrl: u32,    // Ctrl Register
}

#[repr(C)]
struct IoBank{
    pub gpio: [GpioRegs; 48],
    _reserved: [u32; 32],
    pub irqsummary: [u32; 12],
    pub intr: [u32; 6]
}

#[repr(C)]
struct PadsBank{
    pub voltage_select: u32,   // 0x00  bank-wide input threshold
    pub pads: [u32; 48],
    pub swclk: u32,            // 0xc4
    pub swd: u32,              // 0xc8
}

#[repr(C)]
struct Reset{
    pub reset: u32,
    pub wdsel: u32,
    pub reset_done: u32
}

#[repr(C)]
struct Sio{
    pub cpuid:           u32,  // 0x000
    pub gpio_in:         u32,  // 0x004
    pub gpio_in_hi:      u32,  // 0x008
    _reserved:           u32,  // 0x00c  (FIFO_ST is at 0x050; 0x00c is a hole)
    pub gpio_out:        u32,  // 0x010
    pub gpio_out_hi:     u32,  // 0x014
    pub gpio_out_set:    u32,  // 0x018
    pub gpio_out_set_hi: u32,  // 0x01c
    pub gpio_out_clr:    u32,  // 0x020
    pub gpio_out_clr_hi: u32,  // 0x024
    pub gpio_out_xor:    u32,  // 0x028
    pub gpio_out_xor_hi: u32,  // 0x02c
    pub gpio_oe:         u32,  // 0x030
    pub gpio_oe_hi:      u32,  // 0x034
    pub gpio_oe_set:     u32,  // 0x038
    pub gpio_oe_set_hi:  u32,  // 0x03c
    pub gpio_oe_clr:     u32,  // 0x040
    pub gpio_oe_clr_hi:  u32,  // 0x044
    pub gpio_oe_xor:     u32,  // 0x048
    pub gpio_oe_xor_hi:  u32,  // 0x04c
}
