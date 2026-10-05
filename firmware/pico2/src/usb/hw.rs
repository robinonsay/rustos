//! The USB controller's register block and dual-port RAM, as layout
//! structs, plus the two access rules that are easy to get wrong: the
//! split write that hands a buffer to the controller, and which atomic
//! alias each register needs.
//!
//! Everything here is `pub(super)`: only [`crate::usb`] touches the
//! hardware.

use core::arch::asm;

use crate::clocks::{CLK_SYS_HZ, CLK_USB_HZ};
use crate::common::reg::RegAddr;

// --- USBCTRL_REGS ---------------------------------------------------------

/// `USBCTRL_REGS`, base `0x5011_0000` (register list Table 1194,
/// p1157–1158). Only the registers this device-mode driver uses are named;
/// the rest are placeholders that keep the offsets right.
#[repr(C)]
pub(super) struct UsbRegs {
    /// `0x000` `ADDR_ENDP`: bits 6:0 are the address the device answers to
    /// (Table 1195, p1159). 0 after reset; set after SET_ADDRESS.
    pub addr_endp: u32,
    /// `0x004`–`0x03c` `ADDR_ENDP1`–`ADDR_ENDP15`: host mode only.
    _addr_endp_host: [u32; 15],
    /// `0x040` `MAIN_CTRL` (Table 1197, p1160): [`MAIN_CTRL_CONTROLLER_EN`],
    /// `HOST_NDEVICE` (bit 1, 0 = device) and [`MAIN_CTRL_PHY_ISO`].
    pub main_ctrl: u32,
    /// `0x044` `SOF_WR`: host mode only.
    _sof_wr: u32,
    /// `0x048` `SOF_RD`: last frame number seen. Not used.
    _sof_rd: u32,
    /// `0x04c` `SIE_CTRL` (Table 1200, p1161).
    pub sie_ctrl: u32,
    /// `0x050` `SIE_STATUS` (Table 1201, p1162–1163). A mix of read-only
    /// levels and write-1-to-clear event flags; see [`SIE_SETUP_REC`] and
    /// friends.
    pub sie_status: u32,
    /// `0x054` `INT_EP_CTRL`: host mode only.
    _int_ep_ctrl: u32,
    /// `0x058` `BUFF_STATUS` (Table 1203, p1164): one write-1-to-clear bit
    /// per endpoint direction, set when a buffer completes; see
    /// [`buff_bit`].
    pub buff_status: u32,
    /// `0x05c` `BUFF_CPU_SHOULD_HANDLE`: double buffering only.
    _buff_cpu_should_handle: u32,
    /// `0x060` `EP_ABORT` (Table 1205, p1166): same bit layout as
    /// `BUFF_STATUS`. Setting a bit makes the controller NAK every access
    /// to that endpoint and ignore its buffer control word, "in case you
    /// would like to revoke a buffer".
    pub ep_abort: u32,
    /// `0x064` `EP_ABORT_DONE` (Table 1206, p1167): write-1-to-clear; a bit
    /// is set "once an endpoint is idle so the programmer knows it is safe
    /// to modify the buffer control register".
    pub ep_abort_done: u32,
    /// `0x068` `EP_STALL_ARM` (Table 1207, p1168): bit 0 EP0 IN, bit 1
    /// EP0 OUT. Must be set *as well as* the STALL bit in the buffer
    /// control word to stall endpoint 0; the controller clears both bits
    /// whenever a SETUP arrives (§12.7.3.8.1, p1148–1149).
    pub ep_stall_arm: u32,
    /// `0x06c` `NAK_POLL`: host mode only.
    _nak_poll: u32,
    /// `0x070` `EP_STATUS_STALL_NAK`: per-endpoint NAK/STALL flags, only
    /// set if enabled. Not used.
    _ep_status_stall_nak: u32,
    /// `0x074` `USB_MUXING` (Table 1210, p1170).
    pub usb_muxing: u32,
    /// `0x078` `USB_PWR` (Table 1211, p1170).
    pub usb_pwr: u32,
    /// `0x07c` `USBPHY_DIRECT`: direct PHY control. Not used.
    _usbphy_direct: u32,
    /// `0x080` `USBPHY_DIRECT_OVERRIDE`. Not used.
    _usbphy_direct_override: u32,
    /// `0x084` `USBPHY_TRIM`. Not used.
    _usbphy_trim: u32,
    /// `0x088` `LINESTATE_TUNING` (Table 1215, p1173). Deliberately left
    /// at its reset value, as §12.7.2 (p1140) recommends: that value
    /// enables every device-side hardware fix RP2350 added (bits 3–7,
    /// reset 1), including the double read of buffer control words.
    _linestate_tuning: u32,
    /// `0x08c` `INTR` (Table 1216, p1173–1174): raw interrupt flags,
    /// readable whatever `INTE` says. Two of them are change latches this
    /// driver polls: [`INTR_DEV_SUSPEND`] and [`INTR_DEV_CONN_DIS`].
    pub intr: u32,
    /// `0x090` `INTE` (Table 1217, p1174). Kept at 0: this driver is
    /// polled, so the controller's interrupt line is never used.
    pub inte: u32,
}

const _: () = assert!(core::mem::offset_of!(UsbRegs, main_ctrl) == 0x040);
const _: () = assert!(core::mem::offset_of!(UsbRegs, sie_ctrl) == 0x04c);
const _: () = assert!(core::mem::offset_of!(UsbRegs, sie_status) == 0x050);
const _: () = assert!(core::mem::offset_of!(UsbRegs, buff_status) == 0x058);
const _: () = assert!(core::mem::offset_of!(UsbRegs, ep_abort) == 0x060);
const _: () = assert!(core::mem::offset_of!(UsbRegs, ep_stall_arm) == 0x068);
const _: () = assert!(core::mem::offset_of!(UsbRegs, usb_muxing) == 0x074);
const _: () = assert!(core::mem::offset_of!(UsbRegs, usb_pwr) == 0x078);
const _: () = assert!(core::mem::offset_of!(UsbRegs, intr) == 0x08c);
const _: () = assert!(core::mem::offset_of!(UsbRegs, inte) == 0x090);

/// The normal view of the registers.
pub(super) const fn regs() -> *mut UsbRegs {
    RegAddr::USBCTRL_REGS as usize as *mut UsbRegs
}

/// The atomic **set** alias (`+0x2000`, §2.1.3, p27): a write sets exactly
/// the bits written and leaves the others alone.
pub(super) const fn regs_set() -> *mut UsbRegs {
    (RegAddr::USBCTRL_REGS as usize + 0x2000) as *mut UsbRegs
}

/// The atomic **clear** alias (`+0x3000`, §2.1.3, p27).
///
/// Also used for every write-1-to-clear flag (`SIE_STATUS`,
/// `BUFF_STATUS`, `EP_ABORT_DONE`), because that is what the datasheet's
/// own device example does (`usb_hw_clear->sie_status = ...`, §12.7.4.2.3,
/// p1155). The datasheet does not spell out how a clear-alias write meets
/// a write-1-to-clear field; following the reference code is the choice
/// least likely to be wrong. Unverified on hardware.
pub(super) const fn regs_clr() -> *mut UsbRegs {
    (RegAddr::USBCTRL_REGS as usize + 0x3000) as *mut UsbRegs
}

/// `MAIN_CTRL.CONTROLLER_EN`, bit 0.
pub(super) const MAIN_CTRL_CONTROLLER_EN: u32 = 1 << 0;
/// `MAIN_CTRL.PHY_ISO`, bit 2, reset **1**: the PHY is isolated until
/// software clears it — new on RP2350 and the one change existing RP2040
/// code must make (§12.7.2, p1140).
pub(super) const MAIN_CTRL_PHY_ISO: u32 = 1 << 2;

/// `SIE_CTRL.EP0_INT_1BUF`, bit 29: set the EP0 bits of `BUFF_STATUS` for
/// every completed EP0 buffer. EP0 has no endpoint control word, so this
/// is where its "buffer done" reporting is switched on (§12.7.3.7.3,
/// p1147).
pub(super) const SIE_CTRL_EP0_INT_1BUF: u32 = 1 << 29;
/// `SIE_CTRL.PULLUP_EN`, bit 16: the 1.5 kΩ-class pull-up on DP that tells
/// the host a full-speed device is attached.
pub(super) const SIE_CTRL_PULLUP_EN: u32 = 1 << 16;
/// `SIE_CTRL.PULLDOWN_EN`, bit 15, reset **1** on RP2350 (host-mode pull-
/// downs; §12.7.2.2.1, p1141). A device must clear it.
pub(super) const SIE_CTRL_PULLDOWN_EN: u32 = 1 << 15;

/// `SIE_STATUS.BUS_RESET`, bit 19, write-1-to-clear: the host has driven
/// a bus reset.
pub(super) const SIE_BUS_RESET: u32 = 1 << 19;
/// `SIE_STATUS.SETUP_REC`, bit 17, write-1-to-clear: a SETUP packet has
/// been written to DPRAM offset 0.
pub(super) const SIE_SETUP_REC: u32 = 1 << 17;
/// `SIE_STATUS.CONNECTED`, bit 16, read-only, described only as "Device:
/// connected". Writing 1 here clears the [`INTR_DEV_CONN_DIS`] latch.
pub(super) const SIE_CONNECTED: u32 = 1 << 16;
/// `SIE_STATUS.SUSPENDED`, bit 4, read-only: the bus is in the suspend
/// state (no SOF for 3 ms). Writing 1 here clears the
/// [`INTR_DEV_SUSPEND`] latch.
pub(super) const SIE_SUSPENDED: u32 = 1 << 4;

/// `INTR.DEV_SUSPEND`, bit 14: "Set when the device suspend state
/// changes. Cleared by writing to SIE_STATUS.SUSPENDED" (Table 1216,
/// p1174).
pub(super) const INTR_DEV_SUSPEND: u32 = 1 << 14;
/// `INTR.DEV_CONN_DIS`, bit 13: "Set when the device connection state
/// changes. Cleared by writing to SIE_STATUS.CONNECTED" (same table).
pub(super) const INTR_DEV_CONN_DIS: u32 = 1 << 13;

/// `USB_MUXING.TO_PHY`, bit 0 (reset 1): connect the controller to the
/// on-chip PHY.
pub(super) const MUXING_TO_PHY: u32 = 1 << 0;
/// `USB_MUXING.SOFTCON`, bit 3. The datasheet gives this bit no
/// description at all (Table 1210, p1170); it is set here because the
/// datasheet's device example sets it together with `TO_PHY`
/// (§12.7.4.2.1, p1154). Unverified on hardware.
pub(super) const MUXING_SOFTCON: u32 = 1 << 3;

/// `USB_PWR.VBUS_DETECT`, bit 2: the override value for VBUS detect.
pub(super) const PWR_VBUS_DETECT: u32 = 1 << 2;
/// `USB_PWR.VBUS_DETECT_OVERRIDE_EN`, bit 3: use the override value.
pub(super) const PWR_VBUS_DETECT_OVERRIDE_EN: u32 = 1 << 3;

/// `EP_STALL_ARM` bits for both directions of EP0.
pub(super) const STALL_ARM_EP0: u32 = 0b11;

/// Bit of endpoint `n` in `BUFF_STATUS`, `EP_ABORT` and `EP_ABORT_DONE`:
/// `2n` for IN, `2n + 1` for OUT (Table 1203, p1164).
pub(super) const fn buff_bit(n: usize, is_in: bool) -> u32 {
    1 << (2 * n + if is_in { 0 } else { 1 })
}

// --- USBCTRL_DPRAM --------------------------------------------------------

/// One endpoint's pair of words, IN first: the layout of both the endpoint
/// control and the buffer control areas (Table 1191, p1145–1146).
#[repr(C)]
pub(super) struct InOut {
    /// The IN (device-to-host) word.
    pub dir_in: u32,
    /// The OUT (host-to-device) word.
    pub dir_out: u32,
}

/// `USBCTRL_DPRAM`, base `0x5010_0000`: 4 kB of dual-port RAM shared with
/// the controller (§12.7.3.7, p1144; layout Table 1191, p1145–1146).
///
/// Bytes `0x000`–`0x17f` have fixed meanings. Above that the datasheet
/// only says "data buffers"; the three buffers named here are this
/// driver's own allocation, one 64-byte buffer per endpoint direction,
/// each 64-byte aligned because the controller ignores the low six bits of
/// a buffer offset (§12.7.3.7.3 note, p1147).
///
/// # Access rules
///
/// * 8-, 16- and 32-bit accesses all work, unlike most RP2350 registers,
///   and there are **no** atomic set/clear aliases (§12.7.3.7, p1144). So
///   every change here is a plain read or write, never an alias write.
/// * The controller writes a buffer control word back as a 16-bit store
///   to the half belonging to the buffer (§12.7.3.7.1 note, p1145). This
///   driver uses single buffering only, so the upper half is always zero
///   and whole 32-bit writes are safe.
/// * Every access is volatile: the controller changes these bytes behind
///   the compiler's back.
/// * The controller must be out of reset before the DPRAM is touched
///   (§12.7.1.1.3, p1140).
#[repr(C)]
pub(super) struct Dpram {
    /// `0x000`: the eight bytes of the last SETUP packet received.
    pub setup_packet: [u8; 8],
    /// `0x008`–`0x07f`: endpoint control words for EP1–EP15; element `i`
    /// is endpoint `i + 1`. EP0 has none (§12.7.3.7.3, p1147).
    pub ep_ctrl: [InOut; 15],
    /// `0x080`–`0x0ff`: buffer control words for EP0–EP15.
    pub buf_ctrl: [InOut; 16],
    /// `0x100`: EP0 buffer 0, shared by EP0 IN and EP0 OUT.
    pub ep0_buf: [u8; 64],
    /// `0x140`: EP0 buffer 1, used only if EP0 is double-buffered.
    _ep0_buf1: [u8; 64],
    /// `0x180`: EP1 IN (the CDC notification endpoint). Never filled: the
    /// endpoint only ever NAKs.
    pub ep1_in_buf: [u8; 64],
    /// `0x1c0`: EP2 OUT (bulk data from the host).
    pub ep2_out_buf: [u8; 64],
    /// `0x200`: EP2 IN (bulk data to the host).
    pub ep2_in_buf: [u8; 64],
    /// `0x240`–`0xfff`: unused.
    _free: [u8; 0x1000 - 0x240],
}

const _: () = assert!(core::mem::offset_of!(Dpram, ep_ctrl) == 0x008);
const _: () = assert!(core::mem::offset_of!(Dpram, buf_ctrl) == 0x080);
const _: () = assert!(core::mem::offset_of!(Dpram, ep0_buf) == 0x100);
const _: () = assert!(core::mem::offset_of!(Dpram, ep1_in_buf) == 0x180);
const _: () = assert!(core::mem::offset_of!(Dpram, ep2_out_buf) == 0x1c0);
const _: () = assert!(core::mem::offset_of!(Dpram, ep2_in_buf) == 0x200);
const _: () = assert!(core::mem::size_of::<Dpram>() == 0x1000);
const _: () = assert!(core::mem::offset_of!(Dpram, ep1_in_buf).is_multiple_of(64));
const _: () = assert!(core::mem::offset_of!(Dpram, ep2_out_buf).is_multiple_of(64));
const _: () = assert!(core::mem::offset_of!(Dpram, ep2_in_buf).is_multiple_of(64));

/// The DPRAM.
pub(super) const fn dpram() -> *mut Dpram {
    RegAddr::USBCTRL_DPRAM as usize as *mut Dpram
}

// Endpoint control word (Table 1192, p1147).

/// Bit 31: endpoint enabled.
pub(super) const EP_CTRL_ENABLE: u32 = 1 << 31;
/// Bit 29: set the endpoint's `BUFF_STATUS` bit for every completed
/// buffer. Without it the bit never sets, so a polled driver needs it as
/// much as an interrupt-driven one.
pub(super) const EP_CTRL_INT_PER_BUFFER: u32 = 1 << 29;
/// Bits 27:26: endpoint type.
pub(super) const EP_CTRL_TYPE_SHIFT: u32 = 26;
/// Endpoint type 2, bulk.
pub(super) const EP_TYPE_BULK: u32 = 2;
/// Endpoint type 3, interrupt.
pub(super) const EP_TYPE_INTERRUPT: u32 = 3;
// Bits 15:6 hold the buffer's DPRAM offset; the low six bits are ignored,
// so the byte offset itself is written.

// Buffer control word, buffer 0 half (Table 1193, p1148).

/// Bit 15 `FULL`: set by software for IN (the buffer holds data to send);
/// set by the controller on OUT completion (it has filled the buffer).
pub(super) const BC_FULL: u32 = 1 << 15;
/// Bit 13: data PID, DATA0 = 0, DATA1 = 1.
pub(super) const BC_DATA1: u32 = 1 << 13;
/// Bit 11: answer with STALL (on EP0 only together with `EP_STALL_ARM`).
pub(super) const BC_STALL: u32 = 1 << 11;
/// Bit 10 `AVAILABLE`: the controller owns the buffer. Set by software to
/// hand it over, cleared by the controller when the transaction is done.
pub(super) const BC_AVAILABLE: u32 = 1 << 10;
/// Bits 9:0: transfer length — bytes to send for IN, maximum bytes to
/// accept for OUT, and after an OUT completes, bytes actually received.
pub(super) const BC_LEN_MASK: u32 = 0x3ff;

/// No-ops between the two writes of [`write_buffer_control`].
///
/// §12.7.3.7.1 (p1144) asks for enough `clk_sys` cycles that at least one
/// `clk_usb` cycle passes — `ceil(clk_sys / clk_usb)`, 3 in its 125 MHz
/// example, 4 here at 150 MHz. This is twice that, because the Cortex-M33
/// may dual-issue or fold a `nop`, so a `nop` is not guaranteed to cost a
/// whole cycle.
pub(super) const AVAILABLE_DELAY_NOPS: u32 = 2 * CLK_SYS_HZ.div_ceil(CLK_USB_HZ);

/// Write a buffer control word, setting `AVAILABLE` and `STALL` only after
/// the rest of the word has landed.
///
/// The DPRAM is dual-ported across two clock domains, so the controller,
/// on `clk_usb`, can read a word while the processor, on the faster
/// `clk_sys`, is half-way through changing it. If `AVAILABLE` went out in
/// the same write as the length and PID, the controller could see
/// `AVAILABLE` and pair it with the previous packet's length or PID
/// (warning after Table 1193, p1148). The datasheet's procedure
/// (§12.7.3.7.1, p1144) is: write everything but `AVAILABLE`, wait at
/// least one `clk_usb` cycle, then set `AVAILABLE`. The same warning
/// covers `STALL`.
///
/// RP2350 also added a hardware fix, on by default: the controller reads
/// the word twice and only uses it if both reads agree
/// (`LINESTATE_TUNING.DEV_BUFF_CONTROL_DOUBLE_READ_FIX`, Table 1215,
/// p1173), which §12.7.2.2.3 (p1141) says "avoids the need" for the split
/// write. This driver does both.
///
/// The `dsb` makes sure the first write has completed before the delay is
/// counted. The `asm!` blocks are deliberately *not* marked `nomem`, so
/// the compiler also treats them as barriers for memory accesses.
///
/// # Safety
///
/// `reg` must be one of the DPRAM's buffer control words and the caller
/// must own that buffer — `AVAILABLE` clear, or the endpoint aborted
/// through `EP_ABORT`.
pub(super) unsafe fn write_buffer_control(reg: *mut u32, value: u32) {
    let gated = value & (BC_AVAILABLE | BC_STALL);
    unsafe {
        reg.write_volatile(value & !gated);
        if gated != 0 {
            asm!("dsb", options(nostack, preserves_flags));
            for _ in 0..AVAILABLE_DELAY_NOPS {
                asm!("nop", options(nostack, preserves_flags));
            }
            reg.write_volatile(value);
        }
    }
}

/// Copy `src` into a DPRAM packet buffer with volatile byte writes.
///
/// # Safety
///
/// `dst` must point at a DPRAM buffer of at least `src.len()` bytes that
/// the processor currently owns.
pub(super) unsafe fn copy_to_dpram(dst: *mut u8, src: &[u8]) {
    for (i, &b) in src.iter().enumerate() {
        unsafe { dst.add(i).write_volatile(b) };
    }
}

/// Fill `dst` from a DPRAM packet buffer with volatile byte reads.
///
/// # Safety
///
/// `src` must point at a DPRAM buffer of at least `dst.len()` bytes that
/// the processor currently owns.
pub(super) unsafe fn copy_from_dpram(dst: &mut [u8], src: *const u8) {
    for (i, b) in dst.iter_mut().enumerate() {
        *b = unsafe { src.add(i).read_volatile() };
    }
}
