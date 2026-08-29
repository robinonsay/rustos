/* RP2350 / Raspberry Pi Pico 2
 * FLASH: XIP cached window        — §2.2.2, Table 9
 * RAM:   SRAM0-7 + SRAM8-9        — §2.2.3, Tables 10 & 11
 *
 * XIP = execute-in-place: the QMI flash controller presents the external
 * flash chip's contents as ordinary readable memory starting at 0x10000000,
 * so the CPU fetches instructions directly from flash — nothing is copied to
 * RAM first.
 */

/* MEMORY declares the physical regions the linker may place things in.
 * FLASH/RAM are arbitrary labels — only the addresses are real.
 * The (rx)/(rwx) attributes are advisory documentation. They enforce nothing;
 * they only steer where unclaimed ("orphan") sections get dumped.
 */
MEMORY
{
  /* 4M is the DEVICE size (Winbond W25Q32RV on Pico 2), not the window size.
   * QMI chip-select 0 owns a 16 MB window (0x10000000-0x10ffffff, 12.14.1) but
   * drives a fixed 24-bit address phase, so a 4 MB part ALIASES FOUR TIMES
   * inside that window. Using 16M here would let the linker place code into
   * aliases of itself, with no diagnostic. */
  FLASH (rx)  : ORIGIN = 0x10000000, LENGTH = 4M
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 520K
}

/* Records the entry point in the ELF header and, more importantly here, gives
 * --gc-sections a root to trace from. rustc passes --gc-sections by default,
 * so without ENTRY the linker finds nothing reachable and deletes the program.
 */
ENTRY(OnReset)

/* Minimum stack we insist on having room for. Reserved as a real section
 * below so RAM exhaustion becomes a LINK error, not silent corruption. */
_min_stack_size = 8K;

SECTIONS
{
    /* Order here IS flash order: the linker walks these top to bottom, handing
     * out addresses as it goes. The two boot-critical sections must come first.
     * (They were below .text/.rodata; that overlapped once they had content.)
     */

    /* --- .vector_table ---------------------------------------------------
     * Yes, this is the interrupt/exception vector table, and yes, it lives in
     * flash. It is an array of addresses, not code:
     *     word 0      initial stack pointer
     *     word 1      reset handler address
     *     words 2-15  ARM system exceptions (NMI, HardFault, SVCall, ...)
     *     words 16-67 52 external interrupt slots (datasheet 3.2). Note only
     *                 the lower 46 are wired to peripherals; IRQ46-51 are
     *                 SPAREIRQ_IRQ_0..5, "hardwired to zero (never firing)",
     *                 reserved for a core to interrupt itself. The table is
     *                 still 68 entries.
     *   = 68 entries.
     *
     * ORIGIN(FLASH) pins it to offset 0. With no ENTRY_POINT/VECTOR_TABLE item
     * in the metadata block, the bootrom assumes a vector table at the start of
     * the image (datasheet 5.9.3.3): it loads word 0 into SP, word 1 into PC.
     *
     * KEEP(): nothing in Rust *calls* a vector table, so --gc-sections would
     * otherwise delete it as unreachable.
     *
     * ALIGN(512): ARMv8-M requires VTOR alignment of (entries rounded up to a
     * power of two) x 4 = 128 x 4 = 512. Free at the flash base, but stated so
     * it stays true if the table ever moves. Note this is STRICTER than this
     * chip needs -- RP2350's VTOR only implements 128-byte granularity
     * (TBLOFF is bits 31:7, Table 201). The 512 comes from the ARMv8-M
     * architecture, not the RP2350. Keep it: architecture wins.
     */
    .vector_table ORIGIN(FLASH) : ALIGN(512) {
        KEEP(*(.vector_table));
        . = ALIGN(4);   /* RP2350-E25: keep flash segments word-sized */
    } > FLASH

    /* --- .boot_info ------------------------------------------------------
     * RP2350-specific and mandatory. The bootrom scans the first 4 kB of the
     * image for an IMAGE_DEF metadata block and refuses to boot without one:
     * a start marker, an IMAGE_TYPE item tagging the image EXE/ARM/secure, a
     * mandatory LAST item, a link word,
     * and an end marker. This replaces RP2040's 256-byte checksummed
     * second-stage bootloader.
     *
     * Empty until Rust claims it with #[unsafe(link_section = ".boot_info")] —
     * the name must match this one exactly.
     */
    .boot_info : ALIGN(4) {
        KEEP(*(.boot_info));
        . = ALIGN(4);
    } > FLASH

    /* --- .text -----------------------------------------------------------
     * Executable code, run directly from flash over XIP.
     *
     * Read `*(.text .text.*)` as two separate parts:
     *     *        a wildcard over INPUT FILES (every .o and .rlib)
     *     ( ... )  a list of SECTION NAME patterns to take from those files
     * i.e. "from every input file, collect every section named .text or
     * starting with .text., and concatenate them here."
     *
     * The `.text.*` half is not optional. Rust emits one section per function
     * (.text.OnReset, .text.panic, ...) so unused ones can be GC'd. Matching
     * only `.text` would link an empty binary.
     */
    .text : {
        *(.text .text.*)
        . = ALIGN(4);   /* RP2350-E25 */
    } > FLASH

    /* --- .rodata ---------------------------------------------------------
     * Immutable data: `static` items never mutated, string literals, panic
     * messages. Stays in flash and is read in place — the XIP window is
     * read-only by default -- but do not expect a fault on a stray write.
     * XIP_CTRL.CTRL Table 439: "the read-only behaviour is implemented by
     * downgrading writes to reads, so writes will still cause allocation of an
     * address, but have no other effect." A wild write here is a SILENT no-op,
     * which is worse to debug than a fault. (WRITABLE_M0 enables real writes.)
     *
     * `const` items usually appear nowhere: they are inlined at each use site and
     * have no address at all. Only `static` gets a section.
     */
    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(4);   /* RP2350-E25 */
    } > FLASH

    /* --- .data -----------------------------------------------------------
     * Mutable statics with a NON-ZERO initialiser. This is the only section
     * with two addresses, and that is the entire point:
     *
     *     > RAM        sets the VMA (runtime address) — what code refers to
     *     AT > FLASH   sets the LMA (load address)    — where bytes ship
     *
     * The initial value must survive power-off, so it ships in flash; the
     * variable must be writable, so it runs from RAM. Nothing moves it for
     * you — your reset handler copies it, and these symbols are the contract:
     *
     *     __sidata   source      : where the bytes sit in flash  (LMA)
     *     __sdata    destination : where they must end up in RAM (VMA)
     *     __edata    end         : so you know how many to copy
     *
     * `.` is the location counter: the address the linker is currently handing
     * out. `__sdata = .;` captures it before the content and `__edata = .;`
     * after, so the pair brackets the section. `. = ALIGN(4);` rounds it up to
     * the next multiple of 4 so the end lands on a word boundary and the copy
     * loop can move whole u32s.
     *
     * IMPORTANT: these symbols have no VALUE, only an ADDRESS. Declare them in
     * Rust as `unsafe extern "C" { static __sdata: u32; }` and take
     * `&raw const __sdata`. Reading one as a value gives you garbage.
     */
    .data : ALIGN(4) {
        __sdata = .;
        *(.data .data.*)
        . = ALIGN(4);
        __edata = .;
    } > RAM AT > FLASH
    __sidata = LOADADDR(.data);

    .bss (NOLOAD) : ALIGN(4) {
        __sbss = .;
        *(.bss .bss.*)
        . = ALIGN(4);
        __ebss = .;
    } > RAM

    /* Stack reservation. The REAL stack grows DOWN from _stack_top at the top
     * of RAM; this books the same number of bytes at the bottom purely so the
     * linker's region check and --print-memory-usage both account for it.
     * Without it, .bss can grow to 99% of RAM and still link clean. */
    /* ALIGN(8): AAPCS — the Procedure Call Standard for the Arm Architecture,
     * Arm's rules for how compiled functions call each other — requires the
     * stack pointer to be 8-byte aligned at every public interface. */
    .stack (NOLOAD) : ALIGN(8) {
        . = . + _min_stack_size;
    } > RAM
    /DISCARD/ : { *(.ARM.exidx .ARM.exidx.*) }

    /* One past the last valid RAM byte. The M33 stack is full-descending, so
     * the first push lands at _stack_top - 4 and this address is never itself
     * dereferenced -- which matters, because 0x20082000 is outside the decoded
     * SRAM range (10.2.2) and would raise a bus fault if it ever were. */
    _stack_top = ORIGIN(RAM) + LENGTH(RAM);

}
ASSERT(ADDR(.vector_table) == ORIGIN(FLASH),
    "vector table must be at flash base (datasheet 5.9.3.3)");
ASSERT(ADDR(.vector_table) % 512 == 0,
    "vector table violates VTOR alignment");
ASSERT(ADDR(.boot_info) - ORIGIN(FLASH) < 4096,
    "IMAGE_DEF must be in the first 4kB (datasheet 5.9)");
ASSERT(ADDR(.boot_info) % 4 == 0,
    "boot block must start word-aligned (datasheet 5.9.1)");
ASSERT(_stack_top - __ebss >= _min_stack_size,
    "insufficient headroom between .bss and the stack");
