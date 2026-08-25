#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[used]                                          // survives rustc
#[unsafe(link_section = ".boot_info")]           // names the section
static BOOT_INFO: [u32;5] = [
    0xffffded3,
    0x10210142,
    0x000001ff,
    0x00000000,
    0xab123579
];

unsafe extern "C" { static _stack_top: u32; }

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

#[derive(Clone, Copy)]
union Vector {
    handler:   unsafe extern "C" fn(),
    reset:     unsafe extern "C" fn() -> !,
    stack_top: *const u32,
    reserved:  u32,
}

unsafe impl Sync for Vector {}

#[unsafe(no_mangle)] pub extern "C" fn OnReset() -> ! {
    loop{}
}

#[unsafe(no_mangle)] pub extern "C" fn DefaultHandler(){

}

#[unsafe(no_mangle)] pub extern "C" fn OnHardFault(){

}

#[used]
#[unsafe(link_section = ".vector_table")]
static VECTOR_TABLE: [Vector; 68] = {
    let mut t = [Vector { handler: DefaultHandler }; 68];
    t[0] = Vector { stack_top: &raw const _stack_top };
    t[1] = Vector { reset: OnReset };
    t[3] = Vector {handler: OnHardFault};
    t[8] = Vector { reserved: 0};
    t[9] = Vector { reserved: 0};
    t[10] = Vector { reserved: 0};
    t[13] = Vector { reserved: 0};
    t
};
