//! The 52 device-interrupt entries of the vector table (exception numbers
//! 16 to 67, RP2350 datasheet §3.2, Table 94: device interrupt n is exception
//! 16 + n) and the `Vector` word type the table is made of (target builds
//! only).
//!
//! The crate root keeps the table itself (`VECTOR_TABLE`, the core
//! exceptions 0 to 15 and its link section) and fills slots 16 to 67 with
//! [`with_device_interrupts`]. The entries live here, with the interrupt
//! lines they serve, so that the crate root does not grow with them (cwht
//! coding standard CS-18; cwht INSP-096 finding-1).

/// One entry in the vector table.
///
/// A union rather than a plain `u32` so each slot can be written with the
/// value that is actually correct for it while keeping the array homogeneous:
/// slot 0 holds a stack pointer, slot 1 a diverging reset handler, most slots
/// an ordinary handler, and the architecturally reserved slots a literal zero.
/// All variants are word-sized, so the union is a word and the array is a
/// plain table of addresses — which is exactly what the hardware fetches.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union Vector {
    /// An ordinary exception or interrupt handler.
    pub(crate) handler: unsafe extern "C" fn(),
    /// The reset handler. Diverges: there is nothing to return to.
    pub(crate) reset: unsafe extern "C" fn() -> !,
    /// Slot 0 only: the initial stack pointer value.
    pub(crate) stack_top: *const u32,
    /// An architecturally reserved slot, which must read as zero.
    pub(crate) reserved: u32,
}

// SAFETY: `Vector` contains a raw pointer, which is not `Sync`, so the
// compiler will not let a `static` hold one without this. It is sound here
// because the table is immutable, lives in read-only flash, and is only ever
// read by hardware performing vector fetches.
unsafe impl Sync for Vector {}

/// A slot that holds the handler `f`.
const fn handler(f: unsafe extern "C" fn()) -> Vector {
    Vector { handler: f }
}

/// `t` with slot `16 + n` pointing at the handler symbol of device interrupt
/// `n`, for every `n` of [`Irq`](super::Irq) (Table 94). Each symbol defaults
/// to `DefaultHandler` in `link.ld` and is overridden by
/// [`interrupt!`](crate::interrupt). Slots 0 to 15 are returned unchanged.
pub(crate) const fn with_device_interrupts(mut t: [Vector; 68]) -> [Vector; 68] {
    t[16] = handler(TIMER0_IRQ_0);
    t[17] = handler(TIMER0_IRQ_1);
    t[18] = handler(TIMER0_IRQ_2);
    t[19] = handler(TIMER0_IRQ_3);
    t[20] = handler(TIMER1_IRQ_0);
    t[21] = handler(TIMER1_IRQ_1);
    t[22] = handler(TIMER1_IRQ_2);
    t[23] = handler(TIMER1_IRQ_3);
    t[24] = handler(PWM_IRQ_WRAP_0);
    t[25] = handler(PWM_IRQ_WRAP_1);
    t[26] = handler(DMA_IRQ_0);
    t[27] = handler(DMA_IRQ_1);
    t[28] = handler(DMA_IRQ_2);
    t[29] = handler(DMA_IRQ_3);
    t[30] = handler(USBCTRL_IRQ);
    t[31] = handler(PIO0_IRQ_0);
    t[32] = handler(PIO0_IRQ_1);
    t[33] = handler(PIO1_IRQ_0);
    t[34] = handler(PIO1_IRQ_1);
    t[35] = handler(PIO2_IRQ_0);
    t[36] = handler(PIO2_IRQ_1);
    t[37] = handler(IO_IRQ_BANK0);
    t[38] = handler(IO_IRQ_BANK0_NS);
    t[39] = handler(IO_IRQ_QSPI);
    t[40] = handler(IO_IRQ_QSPI_NS);
    t[41] = handler(SIO_IRQ_FIFO);
    t[42] = handler(SIO_IRQ_BELL);
    t[43] = handler(SIO_IRQ_FIFO_NS);
    t[44] = handler(SIO_IRQ_BELL_NS);
    t[45] = handler(SIO_IRQ_MTIMECMP);
    t[46] = handler(CLOCKS_IRQ);
    t[47] = handler(SPI0_IRQ);
    t[48] = handler(SPI1_IRQ);
    t[49] = handler(UART0_IRQ);
    t[50] = handler(UART1_IRQ);
    t[51] = handler(ADC_IRQ_FIFO);
    t[52] = handler(I2C0_IRQ);
    t[53] = handler(I2C1_IRQ);
    t[54] = handler(OTP_IRQ);
    t[55] = handler(TRNG_IRQ);
    t[56] = handler(PROC0_IRQ_CTI);
    t[57] = handler(PROC1_IRQ_CTI);
    t[58] = handler(PLL_SYS_IRQ);
    t[59] = handler(PLL_USB_IRQ);
    t[60] = handler(POWMAN_IRQ_POW);
    t[61] = handler(POWMAN_IRQ_TIMER);
    t[62] = handler(SPAREIRQ_IRQ_0);
    t[63] = handler(SPAREIRQ_IRQ_1);
    t[64] = handler(SPAREIRQ_IRQ_2);
    t[65] = handler(SPAREIRQ_IRQ_3);
    t[66] = handler(SPAREIRQ_IRQ_4);
    t[67] = handler(SPAREIRQ_IRQ_5);
    t
}

// SAFETY: an extern block is unsafe because its declarations are not checked against a
// definition. Each name is defined by `link.ld` as `PROVIDE(<name> = DefaultHandler)`, a
// function of this signature, unless the application defines it with `interrupt!`, whose
// expansion has exactly this signature (`extern "C" fn()`). This crate never calls them; it
// only places their addresses in `VECTOR_TABLE` for the hardware (RP2350 datasheet section
// 3.2, Table 94: device interrupt n is exception 16 + n).
unsafe extern "C" {
    fn TIMER0_IRQ_0();
    fn TIMER0_IRQ_1();
    fn TIMER0_IRQ_2();
    fn TIMER0_IRQ_3();
    fn TIMER1_IRQ_0();
    fn TIMER1_IRQ_1();
    fn TIMER1_IRQ_2();
    fn TIMER1_IRQ_3();
    fn PWM_IRQ_WRAP_0();
    fn PWM_IRQ_WRAP_1();
    fn DMA_IRQ_0();
    fn DMA_IRQ_1();
    fn DMA_IRQ_2();
    fn DMA_IRQ_3();
    fn USBCTRL_IRQ();
    fn PIO0_IRQ_0();
    fn PIO0_IRQ_1();
    fn PIO1_IRQ_0();
    fn PIO1_IRQ_1();
    fn PIO2_IRQ_0();
    fn PIO2_IRQ_1();
    fn IO_IRQ_BANK0();
    fn IO_IRQ_BANK0_NS();
    fn IO_IRQ_QSPI();
    fn IO_IRQ_QSPI_NS();
    fn SIO_IRQ_FIFO();
    fn SIO_IRQ_BELL();
    fn SIO_IRQ_FIFO_NS();
    fn SIO_IRQ_BELL_NS();
    fn SIO_IRQ_MTIMECMP();
    fn CLOCKS_IRQ();
    fn SPI0_IRQ();
    fn SPI1_IRQ();
    fn UART0_IRQ();
    fn UART1_IRQ();
    fn ADC_IRQ_FIFO();
    fn I2C0_IRQ();
    fn I2C1_IRQ();
    fn OTP_IRQ();
    fn TRNG_IRQ();
    fn PROC0_IRQ_CTI();
    fn PROC1_IRQ_CTI();
    fn PLL_SYS_IRQ();
    fn PLL_USB_IRQ();
    fn POWMAN_IRQ_POW();
    fn POWMAN_IRQ_TIMER();
    fn SPAREIRQ_IRQ_0();
    fn SPAREIRQ_IRQ_1();
    fn SPAREIRQ_IRQ_2();
    fn SPAREIRQ_IRQ_3();
    fn SPAREIRQ_IRQ_4();
    fn SPAREIRQ_IRQ_5();
}
