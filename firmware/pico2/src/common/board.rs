use core::sync::atomic::Ordering::Acquire;

use api::define_board;


define_board!{
    Rp2350{
        Rp2350Pins {
            gpio0: 0,
            gpio1: 1,
            gpio2: 2,
            gpio3: 3,
            gpio4: 4,
            gpio5: 5,
            gpio6: 6,
            gpio7: 7,
            gpio8: 8,
            gpio9: 9,
            gpio10: 10,
            gpio11: 11,
            gpio12: 12,
            gpio13: 13,
            gpio14: 14,
            gpio15: 15,
            gpio16: 16,
            gpio17: 17,
            gpio18: 18,
            gpio19: 19,
            gpio20: 20,
            gpio21: 21,
            gpio22: 22,
            // GP23–GP25 and GP29 are committed to on-board functions on the
            // Pico 2 and are not routed to the 40-pin header; named for what
            // the board wired them to rather than as free GPIO.
            smps_ps: 23,     // SMPS power-save select
            vbus_sense: 24,  // high when USB power is present
            led: 25,         // on-board user LED
            gpio26: 26,      // header, also ADC0
            gpio27: 27,      // header, also ADC1
            gpio28: 28,      // header, also ADC2
            vsys_adc: 29,    // ADC3 reads VSYS/3; not on header
        }
    }
}
