//! The watchdog: start, feed, force a reset, and find out why the chip last
//! reset.
//!
//! # What a watchdog is for
//!
//! A countdown timer that resets the chip when it reaches zero. Software
//! that is working reloads ("feeds") it regularly; software that has hung —
//! stuck in a loop, deadlocked, wedged in a fault handler — stops feeding
//! it, and the reset puts the system back into a known state. For a device
//! that keys a transmitter, the known state is the point: a hang must not
//! leave the transmitter keyed.
//!
//! On RP2350 the counter is `CTRL.TIME`, loaded from `LOAD`, and counts down
//! once per microsecond tick (Tables 1247–1248, p1194–1195), up to
//! `0xffffff` µs ≈ 16.7 s ([`MAX_TIMEOUT_US`]).
//!
//! # The tick, and the RP2040 double count
//!
//! The counter is clocked by the `TICKS` WATCHDOG generator, which resets
//! **disabled**; until it runs, an enabled watchdog never counts
//! (§12.9.2, p1191; Table 629, p572). [`Rp2350Watchdog::start`] therefore
//! starts that generator itself. The RP2040's "decrements twice per tick"
//! erratum (RP2040-E1) has no RP2350 counterpart: one tick, one count. Both
//! points are argued in detail in [`crate::timer`].
//!
//! # What a watchdog reset resets
//!
//! The watchdog block itself only produces the event; three registers
//! *outside* it decide how much of the chip the event tears down (§12.9.4,
//! p1192):
//!
//! | Register | Level | Used here |
//! |----------|-------|-----------|
//! | `POWMAN.WDSEL` (Table 489, p471) | chip-level: also reset POWMAN / the switched core | **0**: not used |
//! | `PSM.WDSEL` (Table 531, p499) | system: re-run the power-on state machine from a chosen stage | everything except `ROSC` and `XOSC` |
//! | `RESETS.WDSEL` (Table 535, p505) | subsystem: reset individual peripherals | not needed (see below) |
//!
//! `PSM.WDSEL` resets to 0 — *nothing* selected — so an enabled watchdog
//! with the reset-default scope would expire and reset nothing at all. This
//! driver sets it, as the SDK's `watchdog_enable` does (listing in
//! §12.9.6.1, p1192: "Reset everything apart from ROSC and XOSC"). With
//! the `RESETS` stage selected, the whole reset controller is reset, "which
//! includes every component" (§7.5.2, p503) — every peripheral comes back
//! held in reset, exactly as at power-on — so `RESETS.WDSEL` adds nothing.
//! The two processors (`PROC0`, `PROC1`) are selected too, so both cores
//! restart through the bootrom.
//!
//! The chip-level options in `POWMAN.WDSEL` are left off on purpose. A
//! chip-level reset also resets the watchdog block, and with it `REASON`
//! and the scratch registers (§7.2 note, p493; §12.9.1, p1191) — which
//! would erase the very evidence [`Rp2350Watchdog::reset_reason`] reads.
//!
//! One more register is needed. With POWMAN clocked from `clk_ref` (the
//! default: `SEQ_CFG.USING_FAST_POWCK` resets to 1, Table 490, p472), a
//! watchdog reset that re-runs the `CLOCKS` stage puts a short pulse on
//! `clk_ref` that "may affect POWMAN register state"; `SYSCFG.AUXCTRL`
//! bit 0 moves POWMAN onto the low-power oscillator first, and "must be set
//! before initiating a watchdog reset of the RSM from a stage that includes
//! CLOCKS" (Table 1316, p1252). Since a timeout can happen at any moment
//! after [`start`](Rp2350Watchdog::start), the bit is set there and left
//! set. The SDK's `watchdog_enable` does not do this; the bootrom's reboot
//! routine does. (Unverified on hardware: any side effect of leaving POWMAN
//! on the low-power oscillator for the life of the program.)
//!
//! # Pause bits: never pause
//!
//! `CTRL.PAUSE_DBG0`, `PAUSE_DBG1` and `PAUSE_JTAG` reset to 1: the counter
//! freezes while a debugger halts either core or accesses the bus over
//! JTAG (Table 1247, p1194). [`start`](Rp2350Watchdog::start) clears all
//! three. A debugger attached to a keyer in the field must not be able to
//! hold the transmitter keyed indefinitely by halting the core with the
//! key line asserted. The cost: stepping through code with the watchdog
//! running resets the chip after the timeout, so debug sessions should
//! either not start the watchdog or use a long timeout. (Note also that
//! `TIMER0` *does* pause under debug — see [`crate::timer`].)
//!
//! # Why did the chip reset?
//!
//! Three sources of evidence, all read by [`Rp2350Watchdog::new`] before
//! this driver writes anything:
//!
//! * `WATCHDOG.REASON` (Table 1249, p1195): bit 0 `TIMER` — the counter
//!   reached zero; bit 1 `FORCE` — software wrote `CTRL.TRIGGER`. "Both
//!   bits are zero for the case of a hardware reset." Not cleared by the
//!   PSM-level reset configured here, but cleared by every chip-level
//!   reset and, on RP2350, by a debugger warm reset (`SYSRESETREQ`) of
//!   either core (p1194).
//! * `POWMAN.CHIP_RESET` (Table 488, p468–471): which chip-level reset —
//!   power-on, brown-out, RUN pin, debugger, rescue, glitch detector,
//!   switched-core power-down, chip-level watchdog — happened *last*. It
//!   is not updated by system-level resets, so after a watchdog reset it
//!   still describes the chip-level reset before it.
//! * `WATCHDOG.SCRATCH4`, where [`start`](Rp2350Watchdog::start) leaves a
//!   marker, [`SCRATCH_MARKER`].
//!
//! The marker exists because `REASON` alone cannot separate "this
//! application's watchdog ran out" from "something rebooted the chip on
//! purpose": the bootrom's reboot routine — used by `picotool reboot`, by
//! the BOOTSEL drag-and-drop loader once a UF2 is written, and by the SDK's
//! `watchdog_reboot` — resets the chip by *arming the watchdog timer* with
//! a short delay, not by triggering it, so it too leaves `REASON.TIMER`
//! set. That routine always rewrites `SCRATCH4` (to 0 for a normal reboot,
//! or to the `0xb007c0d3` boot-vector magic). The marker survives the
//! bootrom otherwise: the bootrom clears `SCRATCH4` only when it holds a
//! valid boot-vector request (§5.2.2, Table 450, p368: "Boot type parity
//! is valid — Clear SCRATCH4"; §5.2.4, p372: "If the numbers match, the
//! Bootrom zeroes SCRATCH4"). A sentence a few lines later in §5.2.4
//! speaks of "the bootrom clearing SCRATCH4 each boot"; if that were
//! literally true the marker would never survive and every timeout would
//! be misreported as `WatchdogForced(Reboot)`. Unverified on hardware.
//! Hence:
//!
//! | `REASON` | marker | [`ResetReason`] |
//! |----------|--------|-----------------|
//! | `TIMER` | ours | [`WatchdogTimeout`](ResetReason::WatchdogTimeout) — the application stopped feeding |
//! | `FORCE` | any | [`WatchdogForced(Trigger)`](ResetReason::WatchdogForced) — [`force_reset`](Rp2350Watchdog::force_reset) or other code wrote `TRIGGER` |
//! | `TIMER` | not ours | [`WatchdogForced(Reboot)`](ResetReason::WatchdogForced) — bootrom / picotool / SDK reboot |
//! | 0 | any | [`ChipReset(cause)`](ResetReason::ChipReset) — power-on, RUN pin, debugger, brown-out, ... from `CHIP_RESET` |
//!
//! `new` then clears the marker, so a later reset only reports
//! `WatchdogTimeout` if `start` ran again in the meantime.
//!
//! Limits of the evidence: a debugger warm reset clears `REASON`, so it is
//! reported as `ChipReset` with whatever chip-level cause preceded it; and
//! the datasheet does not itself say which `REASON` bit the bootrom's reboot
//! leaves (the bootrom source arms the timer, hence `TIMER`). Unverified on
//! hardware.
//!
//! # GPIO pins across a watchdog reset
//!
//! The `RESETS` stage puts `IO_BANK0` and `PADS_BANK0` back into reset, but
//! whether a *pad* follows its registers depends on its isolation latch
//! (§9.7, p594–595). Read literally, the datasheet says:
//!
//! * A pad whose `ISO` bit is 0 (transparent — `Rp2350Gpio` clears it when
//!   it configures a pin) returns to the reset state when its registers
//!   are reset: function null, so output disabled; input disabled; pulled
//!   down (§9.3, p587; §9.7, p595: "resetting the PADS register block
//!   returns non-isolated pads to their reset state").
//! * A pad whose `ISO` bit is 1 keeps the output enable, output level and
//!   pulls it had when it was isolated, "even if" its registers are reset
//!   (§9.3, p587). That includes a pin that was being driven **high**.
//! * The `ISO` bits are not reset by a `RESETS`-driven PADS reset; they are
//!   forced to 1 around a power-down and power-up of the switched-core
//!   domain (§9.7, p594–595: "When the switched core power domain powers
//!   back up, all the GPIO ISO bits reset to 1, so the pre-power down state
//!   continues to be maintained"). The latches themselves only clear on
//!   power-on, brown-out, RUN, `CDBGRSTREQ` or rescue reset (§9.7, p594).
//!
//! So with the PSM-level scope used here, a configured output should drop
//! to high-impedance with a weak pull-down; with a scope that resets or
//! powers down the switched core, or across any switched-core power-down,
//! a pin driven high **stays high, latched, until software reconfigures it
//! and clears `ISO`**. The datasheet never states the watchdog case
//! explicitly, and this has not been checked on hardware. The safe rule
//! does not depend on which reading is right: **configure safety-critical
//! outputs, driven to their safe level, first thing after boot** — before
//! the clocks, before USB — and make sure the external circuit treats a
//! high-impedance pin as "off". `Rp2350Gpio`'s output constructor already
//! writes the output low and sets up the pad *before* clearing `ISO`, so
//! the pin goes from its latched state directly to driven-low.
//!
//! # Typical use
//!
//! ```ignore
//! let board = Rp2350::take().unwrap();
//! let mut gpio = Rp2350Gpio::new(board.gpio);
//! let mut key = gpio.output_from_handle(board.pins.gpio15).unwrap(); // safe level first
//! let clocks = Rp2350Clocks::new(board.clocks);
//! let mut watchdog = Rp2350Watchdog::new(board.watchdog, &clocks);   // reads the reason
//! let why = watchdog.reset_reason();
//! watchdog.start(100_000);                                           // 100 ms
//! loop {
//!     // ... work ...
//!     watchdog.feed();
//! }
//! ```
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

use api::device::DeviceHandle;

use crate::clocks::{Rp2350Clocks, CLK_REF_HZ, CLK_SYS_HZ};
use crate::common::reg::RegAddr;
use crate::common::reset::{clr_reset_reg, wait_for_reset_done};
use crate::timer::{start_tick, TickDestination};

/// Longest timeout `LOAD` can hold: `0xffffff` µs, about 16.7 s
/// (Table 1248, p1195).
pub const MAX_TIMEOUT_US: u32 = 0x00ff_ffff;

/// The value [`Rp2350Watchdog::start`] leaves in `SCRATCH4`: ASCII "rust".
/// Any value other than the bootrom's `0xb007c0d3` magic would do.
pub const SCRATCH_MARKER: u32 = 0x7275_7374;

/// The bootrom's watchdog boot-vector magic (§5.2.4, p372).
const BOOTROM_VECTOR_MAGIC: u32 = 0xb007_c0d3;
const _: () = assert!(SCRATCH_MARKER != BOOTROM_VECTOR_MAGIC && SCRATCH_MARKER != 0);

/// Index of the scratch register used for the marker. `SCRATCH4`–`7` are
/// the bootrom's boot-vector registers; a non-magic value in `SCRATCH4`
/// simply means "no vector" to it.
const SCRATCH_MARKER_INDEX: usize = 4;

// --- Register layouts ----------------------------------------------------

/// The `WATCHDOG` block, base `0x400d_8000` (Table 1246, p1194).
#[repr(C)]
struct Watchdog {
    /// `CTRL` (Table 1247, p1194): bit 31 `TRIGGER` (self-clearing), bit 30
    /// `ENABLE`, bits 26/25/24 `PAUSE_DBG1`/`PAUSE_DBG0`/`PAUSE_JTAG`
    /// (reset 1), bits 23:0 `TIME` (read-only, µs left).
    ctrl: u32,
    /// `LOAD` (Table 1248, p1195): write-only, bits 23:0. Writing it reloads
    /// the counter — this *is* the feed.
    load: u32,
    /// `REASON` (Table 1249, p1195): bit 1 `FORCE`, bit 0 `TIMER`,
    /// read-only.
    reason: u32,
    /// `SCRATCH0`–`SCRATCH7` (Table 1250, p1195).
    scratch: [u32; 8],
}

const _: () = assert!(core::mem::offset_of!(Watchdog, load) == 0x04);
const _: () = assert!(core::mem::offset_of!(Watchdog, reason) == 0x08);
const _: () = assert!(core::mem::offset_of!(Watchdog, scratch) == 0x0c);

/// `CTRL.TRIGGER`.
const CTRL_TRIGGER: u32 = 1 << 31;
/// `CTRL.ENABLE`.
const CTRL_ENABLE: u32 = 1 << 30;
/// `CTRL.TIME`.
const CTRL_TIME_MASK: u32 = 0x00ff_ffff;
/// `REASON.FORCE`.
const REASON_FORCE: u32 = 1 << 1;
/// `REASON.TIMER`.
const REASON_TIMER: u32 = 1 << 0;

/// The `PSM` block, base `0x4001_8000` (Table 528, p497).
#[repr(C)]
struct Psm {
    /// `FRCE_ON`: development feature, never written.
    _frce_on: u32,
    /// `FRCE_OFF`: never written.
    _frce_off: u32,
    /// `WDSEL` (Table 531, p499): one bit per PSM stage, 1 = "reset this
    /// stage when the watchdog fires". Bit 0 `PROC_COLD`, 1 `OTP`, 2
    /// `ROSC`, 3 `XOSC`, 4 `RESETS`, 5 `CLOCKS`, 6 `PSM_READY`, 7
    /// `BUSFABRIC`, 8 `ROM`, 9 `BOOTRAM`, 10–19 `SRAM0`–`9`, 20 `XIP`, 21
    /// `SIO`, 22 `ACCESSCTRL`, 23 `PROC0`, 24 `PROC1`. Reset 0.
    wdsel: u32,
}

const _: () = assert!(core::mem::offset_of!(Psm, wdsel) == 0x08);

/// All 25 `PSM.WDSEL` stages.
const PSM_WDSEL_ALL: u32 = (1 << 25) - 1;
/// `PSM.WDSEL.ROSC`.
const PSM_WDSEL_ROSC: u32 = 1 << 2;
/// `PSM.WDSEL.XOSC`.
const PSM_WDSEL_XOSC: u32 = 1 << 3;
/// The scope used here: every stage except the two oscillators, as in the
/// SDK listing of §12.9.6.1 (p1192). The oscillators keep running, so the
/// restart does not wait for the crystal again.
const PSM_WDSEL_SCOPE: u32 = PSM_WDSEL_ALL & !(PSM_WDSEL_ROSC | PSM_WDSEL_XOSC);

/// The parts of `POWMAN` read or written here, base `0x4010_0000`.
#[repr(C)]
struct Powman {
    /// `0x00`–`0x28`, not used.
    _reserved: [u32; 11],
    /// `CHIP_RESET` at `0x2c` (Table 488, p468).
    chip_reset: u32,
    /// `WDSEL` at `0x30` (Table 489, p471). Password-protected.
    wdsel: u32,
}

const _: () = assert!(core::mem::offset_of!(Powman, chip_reset) == 0x2c);
const _: () = assert!(core::mem::offset_of!(Powman, wdsel) == 0x30);

/// POWMAN write password, required in bits 31:16 of every write to a
/// register at offset `0xac` or below; other writes are ignored (§6.4,
/// p455).
const POWMAN_PASSWORD: u32 = 0x5afe << 16;

/// `CHIP_RESET` cause flags (Table 488, p468–471).
const HAD_WATCHDOG_RESET_PSM: u32 = 1 << 28;
const HAD_HZD_SYS_RESET_REQ: u32 = 1 << 27;
const HAD_GLITCH_DETECT: u32 = 1 << 26;
const HAD_SWCORE_PD: u32 = 1 << 25;
const HAD_WATCHDOG_RESET_SWCORE: u32 = 1 << 24;
const HAD_WATCHDOG_RESET_POWMAN: u32 = 1 << 23;
const HAD_WATCHDOG_RESET_POWMAN_ASYNC: u32 = 1 << 22;
const HAD_RESCUE: u32 = 1 << 21;
const HAD_DP_RESET_REQ: u32 = 1 << 19;
const HAD_RUN_LOW: u32 = 1 << 18;
const HAD_BOR: u32 = 1 << 17;
const HAD_POR: u32 = 1 << 16;
/// Any of the four watchdog-initiated chip-level flags.
const HAD_WATCHDOG_ANY: u32 = HAD_WATCHDOG_RESET_PSM
    | HAD_WATCHDOG_RESET_SWCORE
    | HAD_WATCHDOG_RESET_POWMAN
    | HAD_WATCHDOG_RESET_POWMAN_ASYNC;

/// The part of `SYSCFG` used here, base `0x4000_8000` (§12.15.2, p1249).
#[repr(C)]
struct Syscfg {
    /// `0x00`–`0x10`, not used.
    _reserved: [u32; 5],
    /// `AUXCTRL` at `0x14` (Table 1316, p1252). Bit 0: force POWMAN's
    /// clock onto the low-power oscillator.
    auxctrl: u32,
}

const _: () = assert!(core::mem::offset_of!(Syscfg, auxctrl) == 0x14);

/// `SYSCFG.AUXCTRL` bit 0.
const AUXCTRL_POWMAN_CLK_TO_LPOSC: u32 = 1 << 0;

/// `SYSCFG` — bit 20 of `RESETS.RESET` (Table 534, p504).
const RESET_SYSCFG: u32 = 1 << 20;

/// `clk_sys` cycles to wait between moving POWMAN off `clk_ref` and
/// triggering a reset: 64 `clk_ref` cycles. The datasheet gives no figure;
/// the bootrom's reboot routine relies on "approx 5 cycles of clk_ref".
/// Unverified on hardware.
const POWMAN_SWITCH_DELAY_LOOPS: u32 = 64 * (CLK_SYS_HZ / CLK_REF_HZ);

// --- Public types --------------------------------------------------------

/// Why the chip last reset, as determined by [`Rp2350Watchdog::new`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetReason {
    /// The watchdog was not involved (`REASON` = 0): power-on, the RUN
    /// pin, a brown-out, a debugger, ... — the detail comes from
    /// `POWMAN.CHIP_RESET`.
    ChipReset(ChipResetCause),
    /// The watchdog that this driver started ran out: the application
    /// stopped calling [`Rp2350Watchdog::feed`].
    WatchdogTimeout,
    /// A deliberate watchdog reset.
    WatchdogForced(ForcedBy),
}

/// Who forced a [`ResetReason::WatchdogForced`] reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForcedBy {
    /// `CTRL.TRIGGER` was written (`REASON.FORCE`):
    /// [`Rp2350Watchdog::force_reset`], or other code doing the same.
    Trigger,
    /// The watchdog timer expired but this driver had not armed it
    /// (`REASON.TIMER` without [`SCRATCH_MARKER`]): the bootrom's reboot
    /// routine — `picotool reboot`, the end of a UF2 drag-and-drop, the
    /// SDK's `watchdog_reboot` — or some other watchdog user.
    Reboot,
}

/// The most recent chip-level reset, from `POWMAN.CHIP_RESET` (Table 488,
/// p468–471).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipResetCause {
    /// `HAD_POR`: power was applied.
    PowerOn,
    /// `HAD_BOR`: the core supply dropped below the brown-out threshold.
    Brownout,
    /// `HAD_RUN_LOW`: the RUN pin was pulled low (a reset button).
    RunPin,
    /// `HAD_DP_RESET_REQ`: an Arm debugger requested a reset.
    DebuggerReset,
    /// `HAD_RESCUE`: a debugger rescue reset.
    DebuggerRescue,
    /// `HAD_HZD_SYS_RESET_REQ`: a RISC-V debugger system reset.
    RiscvDebuggerReset,
    /// `HAD_GLITCH_DETECT`: the supply glitch detector fired.
    GlitchDetector,
    /// `HAD_SWCORE_PD`: the switched core domain was powered down.
    SwitchedCorePowerDown,
    /// One of the `HAD_WATCHDOG_RESET_*` flags: a watchdog configured (by
    /// someone else; this driver never does) for a chip-level reset.
    WatchdogChipLevel,
    /// No flag set.
    Unknown,
}

/// The raw register values [`Rp2350Watchdog::new`] classified, for logging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResetSnapshot {
    /// `WATCHDOG.REASON`.
    pub reason: u32,
    /// `POWMAN.CHIP_RESET`.
    pub chip_reset: u32,
    /// `WATCHDOG.SCRATCH4` as found at boot.
    pub scratch4: u32,
}

/// Classify a snapshot as described in the module documentation. Pure, so
/// the decision table is easy to check by reading.
const fn classify(s: &ResetSnapshot) -> ResetReason {
    if s.reason & REASON_FORCE != 0 {
        ResetReason::WatchdogForced(ForcedBy::Trigger)
    } else if s.reason & REASON_TIMER != 0 {
        if s.scratch4 == SCRATCH_MARKER {
            ResetReason::WatchdogTimeout
        } else {
            ResetReason::WatchdogForced(ForcedBy::Reboot)
        }
    } else {
        ResetReason::ChipReset(chip_reset_cause(s.chip_reset))
    }
}

/// Decode `CHIP_RESET`. Only one flag is expected; if several are set the
/// most severe (the one that resets the most, per Table 488) wins.
const fn chip_reset_cause(v: u32) -> ChipResetCause {
    if v & HAD_POR != 0 {
        ChipResetCause::PowerOn
    } else if v & HAD_BOR != 0 {
        ChipResetCause::Brownout
    } else if v & HAD_RUN_LOW != 0 {
        ChipResetCause::RunPin
    } else if v & HAD_DP_RESET_REQ != 0 {
        ChipResetCause::DebuggerReset
    } else if v & HAD_RESCUE != 0 {
        ChipResetCause::DebuggerRescue
    } else if v & HAD_WATCHDOG_ANY != 0 {
        ChipResetCause::WatchdogChipLevel
    } else if v & HAD_SWCORE_PD != 0 {
        ChipResetCause::SwitchedCorePowerDown
    } else if v & HAD_GLITCH_DETECT != 0 {
        ChipResetCause::GlitchDetector
    } else if v & HAD_HZD_SYS_RESET_REQ != 0 {
        ChipResetCause::RiscvDebuggerReset
    } else {
        ChipResetCause::Unknown
    }
}

// --- Driver --------------------------------------------------------------

/// The watchdog driver. See the [module documentation](self) for the
/// reset scope, the pause-bit policy and the reset-reason logic.
pub struct Rp2350Watchdog {
    /// What [`feed`](Self::feed) writes to `LOAD`; 0 until started.
    load: u32,
    snapshot: ResetSnapshot,
    reason: ResetReason,
}

impl Rp2350Watchdog {
    /// Capture and classify the reset reason, then clear this driver's
    /// marker. Does not start, stop or reconfigure the watchdog.
    ///
    /// Call it early: nothing in this crate clears `REASON` or
    /// `CHIP_RESET`, but `SCRATCH4` is shared with the bootrom's reboot
    /// mechanism. `&Rp2350Clocks` is required because
    /// [`start`](Self::start) relies on `clk_ref` being the 12 MHz crystal.
    pub fn new(_handle: DeviceHandle<Rp2350Watchdog>, _clocks: &Rp2350Clocks) -> Self {
        let wd = RegAddr::WATCHDOG as usize as *mut Watchdog;
        let powman = RegAddr::POWMAN as usize as *const Powman;
        let snapshot = unsafe {
            let scratch4 = &raw mut (*wd).scratch[SCRATCH_MARKER_INDEX];
            let snapshot = ResetSnapshot {
                reason: (&raw const (*wd).reason).read_volatile(),
                chip_reset: (&raw const (*powman).chip_reset).read_volatile(),
                scratch4: scratch4.read_volatile(),
            };
            if snapshot.scratch4 == SCRATCH_MARKER {
                scratch4.write_volatile(0);
            }
            snapshot
        };
        Self { load: 0, snapshot, reason: classify(&snapshot) }
    }

    /// Why the chip last reset.
    pub fn reset_reason(&self) -> ResetReason {
        self.reason
    }

    /// The raw registers behind [`reset_reason`](Self::reset_reason).
    pub fn reset_snapshot(&self) -> ResetSnapshot {
        self.snapshot
    }

    /// Start (or restart with a new timeout) the watchdog. The chip resets
    /// unless [`feed`](Self::feed) is called at least every `timeout_us`
    /// microseconds. `timeout_us` is clamped to `1..=`[`MAX_TIMEOUT_US`];
    /// the value actually used is returned.
    ///
    /// Sequence, after the SDK listing in §12.9.6.1 (p1192) plus the two
    /// RP2350 additions argued in the module docs:
    ///
    /// 1. `CTRL` = 0: stop counting while reconfiguring (and clear the
    ///    pause bits).
    /// 2. Reset scope: `SYSCFG` out of reset, `PSM.WDSEL` = all but the
    ///    oscillators, `POWMAN.WDSEL` = 0, `SYSCFG.AUXCTRL` bit 0 set.
    /// 3. Start the `TICKS` WATCHDOG generator at 12 cycles = 1 µs.
    /// 4. `SCRATCH4` = [`SCRATCH_MARKER`].
    /// 5. `LOAD` = timeout.
    /// 6. `CTRL` = `ENABLE`, pause bits clear.
    pub fn start(&mut self, timeout_us: u32) -> u32 {
        let load = timeout_us.clamp(1, MAX_TIMEOUT_US);
        let wd = RegAddr::WATCHDOG as usize as *mut Watchdog;
        unsafe {
            let ctrl = &raw mut (*wd).ctrl;
            ctrl.write_volatile(0);
            configure_reset_scope();
            start_tick(TickDestination::Watchdog);
            (&raw mut (*wd).scratch[SCRATCH_MARKER_INDEX]).write_volatile(SCRATCH_MARKER);
            (&raw mut (*wd).load).write_volatile(load);
            ctrl.write_volatile(CTRL_ENABLE);
        }
        self.load = load;
        load
    }

    /// Reload the counter with the timeout given to
    /// [`start`](Self::start). Does nothing before `start`.
    ///
    /// Feed from the place that proves the application is healthy — the
    /// main loop after its work is done — not from a timer interrupt that
    /// keeps running when the main loop hangs.
    pub fn feed(&self) {
        if self.load == 0 {
            return;
        }
        let wd = RegAddr::WATCHDOG as usize as *mut Watchdog;
        unsafe { (&raw mut (*wd).load).write_volatile(self.load) };
    }

    /// Microseconds left before the watchdog fires (`CTRL.TIME`).
    pub fn time_remaining_us(&self) -> u32 {
        let wd = RegAddr::WATCHDOG as usize as *const Watchdog;
        unsafe { (&raw const (*wd).ctrl).read_volatile() & CTRL_TIME_MASK }
    }

    /// Whether the watchdog is counting (`CTRL.ENABLE`).
    pub fn is_running(&self) -> bool {
        let wd = RegAddr::WATCHDOG as usize as *const Watchdog;
        unsafe { (&raw const (*wd).ctrl).read_volatile() & CTRL_ENABLE != 0 }
    }

    /// Reset the chip now, through the watchdog. Never returns.
    ///
    /// Configures the same reset scope as [`start`](Self::start) — so this
    /// works even if the watchdog was never started — gives POWMAN time to
    /// move off `clk_ref`, then writes `CTRL.TRIGGER`. `TRIGGER` does not
    /// need `ENABLE` (the SDK's `watchdog_enable(0, ...)` path in
    /// §12.9.6.1, p1192–1193, triggers with `ENABLE` clear). The next boot
    /// reports [`ResetReason::WatchdogForced`]`(`[`ForcedBy::Trigger`]`)`.
    pub fn force_reset(&mut self) -> ! {
        let wd = RegAddr::WATCHDOG as usize as *mut Watchdog;
        unsafe {
            configure_reset_scope();
            for _ in 0..POWMAN_SWITCH_DELAY_LOOPS {
                core::hint::spin_loop();
            }
            let ctrl = &raw mut (*wd).ctrl;
            ctrl.write_volatile(ctrl.read_volatile() | CTRL_TRIGGER);
        }
        loop {
            core::hint::spin_loop();
        }
    }
}

/// Make a watchdog event re-run the power-on state machine from its first
/// non-oscillator stage, and prepare POWMAN for it. See "What a watchdog
/// reset resets" in the module docs.
unsafe fn configure_reset_scope() {
    let psm = RegAddr::PSM as usize as *mut Psm;
    let powman = RegAddr::POWMAN as usize as *mut Powman;
    let syscfg = RegAddr::SYSCFG as usize as *mut Syscfg;
    unsafe {
        // SYSCFG is a RESETS-controlled block (Table 534, p504). Release it
        // if something left it in reset; never assert its reset, because
        // its other registers (e.g. the GPIO input synchroniser bypass)
        // may be in use.
        clr_reset_reg(!RESET_SYSCFG);
        wait_for_reset_done(RESET_SYSCFG);

        (&raw mut (*psm).wdsel).write_volatile(PSM_WDSEL_SCOPE);
        (&raw mut (*powman).wdsel).write_volatile(POWMAN_PASSWORD);
        let auxctrl = &raw mut (*syscfg).auxctrl;
        auxctrl.write_volatile(auxctrl.read_volatile() | AUXCTRL_POWMAN_CLK_TO_LPOSC);
    }
}
