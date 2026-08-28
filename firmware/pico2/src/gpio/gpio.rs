use core::hint::spin_loop;

use crate::gpio::{IoBank, PadsBank, Reset, Sio};
use crate::common::reg::RegAddr;

unsafe fn reset_gpio(){
    // Bit for IOBANK
    const IOBANK_RESET_BIT:u8 = 6;
    // Bit for Pad
    const PADBANK_RESET_BIT:u8 = 9;
    // IO Pad bitmask
    const IO_PAD_BITMASK: u32 = 1 << IOBANK_RESET_BIT | 1 << PADBANK_RESET_BIT;
    // Create pointer to reset addresses
    let reset_addr = RegAddr::RESET as usize as *mut Reset;
    unsafe{
        // read current registers
        let reset = &raw mut (*reset_addr).reset;
        let current = reset.read_volatile();
        // reset IO and PAD
        reset.write_volatile(current & !IO_PAD_BITMASK);
        let reset_done = &raw const (*reset_addr).reset_done;
        // Wait for it to be done
        while reset_done.read_volatile() & IO_PAD_BITMASK != IO_PAD_BITMASK
        {}
    }
}

unsafe fn configure_gpio_pin_out(pin: usize)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe{
        let gpio_out_clr = &raw mut (*sio_addr).gpio_out_clr;
        gpio_out_clr.write_volatile(1 << pin);
        let gpio_oe_set = &raw mut (*sio_addr).gpio_oe_set;
        gpio_oe_set.write_volatile(1 << pin);
        let pad= &raw mut (*pads_addr).pads[pin];
        let mut current_pad = pad.read_volatile();
        const IE: u8 = 6;
        const OD: u8 = 7;
        current_pad &= !(1 << OD);
        current_pad |= 1 << IE;
        pad.write_volatile( current_pad);
        let io_ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        const SIO: u32 = 5;
        io_ctrl.write_volatile(SIO);
        let mut current_pad = pad.read_volatile();
        const ISO: u8 = 8;
        current_pad &= !(1 << ISO);
        pad.write_volatile(current_pad);
    }
}

unsafe fn toggle_gpio_pin(pin: usize)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    unsafe{
        let toggle = &raw mut (*sio_addr).gpio_out_xor;
        toggle.write_volatile(1 << pin);
    }
}

pub unsafe fn gpio_demo(){
    unsafe{
        reset_gpio();
        configure_gpio_pin_out(25);
    }
    loop{
        unsafe{
            toggle_gpio_pin(25);
        }
        for _ in 0..500_000 {spin_loop();}
    }
}
