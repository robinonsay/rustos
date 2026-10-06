//! Chip-independent USB device logic: a CDC-ACM ("USB serial") function.
//!
//! Everything in this module is pure computation on bytes. It never touches
//! a register, so it compiles for the host and every byte it produces is
//! pinned down by the unit tests at the bottom of each file. The chip driver
//! (`pico2::usb` for the RP2350) owns the hardware half: it moves packets
//! between the USB controller and these functions, and applies the
//! [`control::Effect`]s they ask for.
//!
//! # A two-minute tour of USB, as far as this code needs it
//!
//! A full-speed USB device is a set of numbered **endpoints**. Endpoint 0 is
//! the **control endpoint**, which every device has and through which the
//! host discovers and configures it. The other endpoints exist only once the
//! host has chosen a **configuration**. Each non-zero endpoint is one-way:
//! an **IN** endpoint carries data to the host and an **OUT** endpoint from
//! it, and the direction is bit 7 of the endpoint address (`0x82` is
//! endpoint 2 IN, `0x02` is endpoint 2 OUT) — USB 2.0 §9.6.6.
//!
//! The host is always in charge. It sends a token naming an endpoint and the
//! device answers with data, an ACK, a NAK ("not ready, ask again") or a
//! STALL ("this request is not supported" / "this endpoint is halted"). A
//! device driver therefore never *sends* anything on its own initiative: it
//! *arms* a buffer and the hardware hands it over the next time the host
//! asks.
//!
//! ## Control transfers (endpoint 0)
//!
//! A control transfer has up to three stages (USB 2.0 §8.5.3):
//!
//! 1. **Setup** — the host sends an 8-byte [`setup::SetupPacket`] saying what
//!    it wants. The hardware always accepts it.
//! 2. **Data** (optional) — `wLength` bytes move in the direction given by
//!    bit 7 of `bmRequestType`, split into packets of at most 64 bytes.
//! 3. **Status** — a zero-length packet in the *opposite* direction to the
//!    data stage (or IN, if there was no data stage) that tells the other
//!    side the request completed. A STALL here, or instead of the data
//!    stage, means "request error".
//!
//! [`control::ControlHandler::setup`] decides which of these shapes a
//! request takes and returns it as a [`control::ControlAction`]; the driver
//! performs the stages.
//!
//! ## Data toggles
//!
//! Every data packet carries a PID of DATA0 or DATA1 that alternates per
//! endpoint, so that a packet re-sent after a lost ACK can be recognised as
//! a duplicate (USB 2.0 §8.6). The first data packet after a SETUP is always
//! DATA1, and so is the status packet. On bulk and interrupt endpoints the
//! toggle starts at DATA0 when the endpoint is configured, and returns to
//! DATA0 whenever the host clears a halt on it (USB 2.0 §9.4.5,
//! ClearFeature(ENDPOINT_HALT)). [`control::Effect`] tells the driver when
//! to do that.
//!
//! ## Short packets and zero-length packets
//!
//! USB has no "end of message" marker. A transfer ends when a packet arrives
//! that is *shorter than the endpoint's maximum packet size*, or when the
//! receiver has all the bytes it asked for (USB 2.0 §5.5.3 for control,
//! §5.8.3 for bulk). So if a message happens to be an exact multiple of 64
//! bytes, the sender must follow it with a zero-length packet (ZLP), or the
//! host keeps waiting for more. [`control_in_needs_zlp`] and
//! [`bulk_in_needs_zlp`] encode the two forms of that rule.
//!
//! # CDC-ACM
//!
//! "Communications Device Class, Abstract Control Model" is the USB class
//! that every desktop OS treats as a serial port with no driver install:
//! Linux binds `cdc_acm` (`/dev/ttyACM0`), macOS its built-in CDC driver
//! (`/dev/cu.usbmodem*`), and Windows 10 and later `usbser.sys` (a COM
//! port). It uses two interfaces: a *communication* interface carrying
//! modem-control requests (baud rate, DTR/RTS) on endpoint 0 plus a
//! notification endpoint, and a *data* interface with one bulk IN and one
//! bulk OUT endpoint that carry the byte stream. The layout used here is in
//! [`descriptor`].
//!
//! # Module map
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`setup`] | Decoding the 8-byte setup packet; standard request, descriptor-type and feature codes. |
//! | [`descriptor`] | The configurable device descriptor, the fixed CDC-ACM configuration descriptor, string descriptors. |
//! | [`cdc_acm`] | Class request codes, line coding, control-line state. |
//! | [`control`] | [`control::ControlHandler`]: the endpoint-0 request state machine. |
//! | [`ring`] | [`ring::ByteRing`], the fixed-capacity byte FIFO used for the serial stream. |
//!
//! # References
//!
//! * *Universal Serial Bus Specification, Revision 2.0* (USB-IF, 2000),
//!   chapters 5, 8 and 9 — cited as "USB 2.0 §n".
//! * *USB Interface Association Descriptor Device Class Code and Use Model*
//!   (USB-IF, 2003) — cited as "IAD ECN".
//! * *USB Class Definitions for Communications Devices, Revision 1.2* —
//!   cited as "CDC 1.2 §n".
//! * *USB Communications Class Subclass Specification for PSTN Devices,
//!   Revision 1.2* — cited as "PSTN 1.2 §n". ACM is defined here.

pub mod setup;
pub mod descriptor;
pub mod cdc_acm;
pub mod control;
pub mod ring;

/// Does an IN data stage of a control transfer need a trailing zero-length
/// packet?
///
/// `reply_len` is how many bytes the device is sending (already clipped to
/// the host's `wLength`), `w_length` is what the host asked for, and
/// `max_packet` is endpoint 0's maximum packet size.
///
/// USB 2.0 §5.5.3: the data stage ends when the device has sent exactly
/// `wLength` bytes, *or* sends a packet shorter than `max_packet`. If the
/// device is sending *less* than was asked for, and that amount is a whole
/// number of full packets (including zero), the last packet it sent was
/// full-sized, so the host cannot tell the reply has ended — a ZLP is what
/// tells it. If the reply is exactly `wLength` bytes no ZLP is sent, even
/// when it is a multiple of `max_packet`: the host stops on the byte count.
///
/// ```
/// use api::usb::control_in_needs_zlp;
/// assert!(!control_in_needs_zlp(18, 64, 64));   // short packet ends it
/// assert!(control_in_needs_zlp(64, 255, 64));   // full packet, host wanted more
/// assert!(!control_in_needs_zlp(64, 64, 64));   // exactly wLength: no ZLP
/// assert!(control_in_needs_zlp(0, 7, 64));      // nothing to send: the ZLP is the reply
/// ```
pub const fn control_in_needs_zlp(reply_len: usize, w_length: usize, max_packet: usize) -> bool {
    reply_len < w_length && reply_len.is_multiple_of(max_packet)
}

/// Should a bulk IN endpoint send a zero-length packet now?
///
/// `last_packet_len` is the length of the packet that just completed,
/// `max_packet` the endpoint's maximum packet size, and `more_queued`
/// whether any bytes are waiting to follow it.
///
/// A serial stream has no message boundaries of its own; a host read
/// completes when a short packet arrives (USB 2.0 §5.8.3). If the stream
/// pauses right after a full-sized packet, the host may be left holding a
/// partially filled read buffer until the next byte is typed. Sending one
/// ZLP at that moment ends the host's transfer so it sees the data
/// immediately. If more bytes are queued they will follow, and the next
/// packet ends the transfer instead.
///
/// ```
/// use api::usb::bulk_in_needs_zlp;
/// assert!(bulk_in_needs_zlp(64, 64, false));
/// assert!(!bulk_in_needs_zlp(64, 64, true));
/// assert!(!bulk_in_needs_zlp(10, 64, false));
/// assert!(!bulk_in_needs_zlp(0, 64, false));   // a ZLP itself needs no ZLP
/// ```
pub const fn bulk_in_needs_zlp(last_packet_len: usize, max_packet: usize, more_queued: bool) -> bool {
    !more_queued && last_packet_len != 0 && last_packet_len == max_packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_zlp_rule() {
        // Shorter than one packet: the short packet itself ends the stage.
        assert!(!control_in_needs_zlp(18, 64, 64));
        // Two full packets of a 255-byte request: needs a ZLP.
        assert!(control_in_needs_zlp(128, 255, 64));
        // 75-byte configuration descriptor: 64 + 11, the 11 is short.
        assert!(!control_in_needs_zlp(75, 255, 64));
        // Host asked for exactly what we have: no ZLP even at a multiple.
        assert!(!control_in_needs_zlp(128, 128, 64));
        // Empty reply to a request with a data stage.
        assert!(control_in_needs_zlp(0, 2, 64));
    }

    #[test]
    fn bulk_zlp_rule() {
        assert!(bulk_in_needs_zlp(64, 64, false));
        assert!(!bulk_in_needs_zlp(64, 64, true));
        assert!(!bulk_in_needs_zlp(63, 64, false));
        assert!(!bulk_in_needs_zlp(0, 64, false));
    }
}
