# UART: Operation Modes & Initialisation

[Back to UART index](index.md) | [Back to ICD index](../index.md)

## 12.1.3.2.1. Fractional Baud-Rate Divider

The 22-bit divisor is split into:

- 16-bit integer part: `UARTIBRD` (offset 0x024).
- 6-bit fractional part: `UARTFBRD` (offset 0x028).

Definition:

```
BaudRateDivisor = UARTCLK / (16 × BaudRate)
                = BRDI + BRDF / 64
```

The fractional value is computed as:

```
m = floor(F × 64 + 0.5)    where F is the fractional part
```

> **Important:** After writing `UARTIBRD` and `UARTFBRD`, software must
> perform a (dummy) write to `UARTLCR_H` to latch the new divisors into
> the baud generator. This is a quirk of the PL011.

### Worked Example: 115 200 baud, UARTCLK = 125 MHz

```
BaudRateDivisor = 125e6 / (16 × 115200) ≈ 67.817
BRDI = 67
F    = 0.817
BRDF = floor(0.817 × 64 + 0.5) = 52
Effective divisor = 67 + 52/64 = 67.8125
Effective baud    = 125e6 / (16 × 67.8125) ≈ 115 207
Error             = (115207 − 115200) / 115200 ≈ 0.006 %
```

### FT1 Example: 9600 baud, UARTCLK = 125 MHz

```
BaudRateDivisor = 125e6 / (16 × 9600) ≈ 813.802
BRDI = 813
BRDF = floor(0.802 × 64 + 0.5) = 51
```

## 12.1.3.2.2. Data Transmission

After enabling the UART:

- Writes to `UARTDR` push bytes into the TX FIFO.
- TX FIFO is drained by the transmit shifter and serialised.
- `UARTFR.BUSY` asserts as soon as the FIFO is non-empty and remains high
  until the last bit (including stop bits) leaves the shifter.

## 12.1.3.2.3-4. Errors

| Error | Stored Where | Cleared By |
|-------|--------------|------------|
| Framing error | RX FIFO bit 8 (per-byte) | Reading the byte; then write `UARTECR` |
| Parity error  | RX FIFO bit 9 | Same |
| Break error   | RX FIFO bit 10 | Same |
| Overrun error | RX FIFO bit 11 (next-good byte's slot) | Read byte; write `UARTECR` |

When the receive FIFO is full and another byte arrives, the new byte is
discarded (not written). The overrun flag rides with the next byte that
does fit.

## 12.1.3.2.5. Disabling the FIFOs

`UARTLCR_H.FEN = 0` collapses the FIFOs into 1-byte holding registers.
Useful for low-latency mode but disables burst DMA.

## 12.1.3.2.6. Loopback

Set `UARTCR.LBE` to internally route TXD → RXD for self-test.

## 12.1.7. Initialisation Sequence

Recommended SDK ordering (`uart_init`):

1. De-assert peripheral reset (RESETS block).
2. Ensure `clk_peri` is enabled and stable.
3. **Disable** the UART (`UARTCR.UARTEN=0`) before changing `UARTLCR_H`,
   `UARTIBRD`, `UARTFBRD`. (Optional but safer than mid-stream changes.)
4. Compute and write `UARTIBRD` and `UARTFBRD`.
5. Write `UARTLCR_H` with desired format **and** `FEN=1` to enable FIFOs.
   (This LCR_H write also latches the baud divisors.)
6. Set FIFO interrupt levels (`UARTIFLS`) if non-default.
7. Enable interrupts you intend to use (`UARTIMSC`).
8. Enable DMA if used (`UARTDMACR`).
9. Set `UARTCR = UARTEN | TXE | RXE`.
10. Configure GPIO pins for the chosen UART instance (`gpio_set_function`,
    function `0x02`).

### Pseudo-code (paraphrased SDK)

```c
uint uart_init(uart_inst_t *uart, uint baudrate) {
    uart_reset(uart);
    uart_unreset(uart);

    uart_set_baudrate(uart, baudrate);

    // Format: 8N1, FIFO enabled
    hw_write_masked(&uart_get_hw(uart)->lcr_h,
        ((8u - 5u) << UARTLCR_H_WLEN_LSB)
      | (0u << UARTLCR_H_STP2_LSB)
      | (0u << UARTLCR_H_PEN_LSB)
      | UARTLCR_H_FEN_BITS,
        UARTLCR_H_WLEN_BITS | UARTLCR_H_STP2_BITS
      | UARTLCR_H_PEN_BITS  | UARTLCR_H_EPS_BITS
      | UARTLCR_H_FEN_BITS);

    // Enable
    uart_get_hw(uart)->cr =
        UARTCR_UARTEN_BITS | UARTCR_TXE_BITS | UARTCR_RXE_BITS;
    // Always enable DMA request signals (harmless if no DMA listener)
    uart_get_hw(uart)->dmacr =
        UARTDMACR_TXDMAE_BITS | UARTDMACR_RXDMAE_BITS;

    return baud;
}
```

### Baud-Rate Setter (paraphrased SDK)

```c
uint uart_set_baudrate(uart_inst_t *uart, uint baudrate) {
    uint32_t bdiv = (8 * uart_clock_get_hz(uart) / baudrate) + 1;
    uint32_t ibrd = bdiv >> 7;
    uint32_t fbrd;
    if      (ibrd == 0)     { ibrd = 1;     fbrd = 0; }
    else if (ibrd >= 65535) { ibrd = 65535; fbrd = 0; }
    else                    {               fbrd = (bdiv & 0x7f) >> 1; }

    uart_get_hw(uart)->ibrd = ibrd;
    uart_get_hw(uart)->fbrd = fbrd;

    // Latch divisors via dummy LCR_H write
    uart_write_lcr_bits_masked(uart, 0, 0);

    return (4 * uart_clock_get_hz(uart)) / (64 * ibrd + fbrd);
}
```

## TX/RX Operations Modes

### Transmit Mode

- Software writes byte to `UARTDR` (write data path).
- TX FIFO interrupt fires when level drops below `TXIFLSEL`.
- For DMA-driven TX: enable `UARTDMACR.TXDMAE` and configure the DMA
  channel with `DREQ_UARTn_TX`.

### Receive Mode

- Software reads `UARTDR` to pop a byte. Status bits in [11:8] indicate
  per-byte errors.
- RX interrupt (`UARTRXINTR`) asserts when the RX FIFO fills above
  `RXIFLSEL`. Cleared by reading the FIFO down below the level.
- Receive timeout (`UARTRTINTR`) asserts when the RX FIFO has data but no
  new character has arrived for 32 bit-times — useful for closing a
  variable-length packet.

## See Also

- [`03_interrupts.md`](03_interrupts.md) — All interrupt sources.
- [`04_registers.md`](04_registers.md) — Register-level details.
- Source PDF Figure 66 for DMA transfer waveforms.
