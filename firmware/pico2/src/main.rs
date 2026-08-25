#![no_std]
#![no_main]

use core::{panic::PanicInfo, ptr::copy_nonoverlapping};

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
unsafe extern "C" {
    static __sidata: u32;  static __sdata: u32;  static __edata: u32;
    static __sbss:   u32;  static __ebss:  u32;
}

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

/// Cortex-M33 private peripheral block. Datasheet 3.7.5: "The Arm Cortex-M33
/// registers start at a base address of 0xe0000000, defined as PPB_BASE".
const PPB_BASE: usize = 0xE000_0000;

/// Coprocessor Access Control Register. Datasheet 3.7, offset 0x0ed88.
const CPACR: *mut u32 = (PPB_BASE + 0x0ED88) as *mut u32;

/// The VTOR (Vector Table Offset Register)
const VTOR: *mut u32 = (PPB_BASE + 0x0ED08) as *mut u32;

/// Full access (0b11) for CP10 and CP11 — together these are the FP extension.
/// Both must hold the same value or the result is UNKNOWN (Table 229).
const CPACR_FPU_FULL: u32 = (0b11 << 20) | (0b11 << 22);   // == 0x00F0_0000

#[inline]
unsafe fn enable_fpu() { unsafe {
    let current = CPACR.read_volatile();      // READ
    let updated = current | CPACR_FPU_FULL;   // MODIFY — preserves CP0/CP4/CP5/CP7
    CPACR.write_volatile(updated);            // WRITE
    core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
}}

#[inline]
unsafe fn reset_vtor() {
    unsafe{
        VTOR.write_volatile(&raw const VECTOR_TABLE as u32);
    }
}

#[unsafe(no_mangle)] pub extern "C" fn OnReset() -> ! {
    unsafe{
        enable_fpu();
        reset_vtor();
    }
    loop{}
}

#[unsafe(no_mangle)] pub extern "C" fn DefaultHandler(){

}

#[unsafe(no_mangle)] pub extern "C" fn OnHardFault(){
    loop{}
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
