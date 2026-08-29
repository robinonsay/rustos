# Superseded — revision A (2026-08-25)

**Nothing in this directory is current. Do not follow it.**

These seven files are revision A of the RP2350 bare-metal tutorial. They were
replaced on 2026-08-28 by revision B, which lives one directory up as
`01_setup_and_workspace.md` through `09_gpio_reference.md`.

Start at [`../index.md`](../index.md). It is the only entry point.

## Why they are not merely older

Revision A numbered its chapters 01-07 and revision B numbers its chapters
01-09, so the two sets collide on every number from 01 to 07 — a reader who
opened `01_*.md` in the parent directory had a coin-flip chance of landing in
`01_linker_scripts.md`, which contains no toolchain setup at all. That is why
they were moved rather than left in place.

Revision A is also wrong in ways revision B fixes. It documents helpers that
do not exist in the tree (`with_funcsel`, `FUNCSEL_MASK`, `io_bank0_ctrl_offset`,
a `Pin` type with a `Drop` impl, host tests of register arithmetic), it states
the GPIO bring-up order as RESETS → FUNCSEL → OE → OUT → IE/OD → ISO when the
firmware actually does OUT → OE → IE/OD → FUNCSEL → ISO, and several of its
"verified output" blocks are from a build that no longer exists.

They are kept only so the revision history is not lost.

| Revision A file | Superseded by |
|---|---|
| `01_linker_scripts.md` | `../02_linker_scripts.md` |
| `02_memory_map.md` | `../03_memory_map.md` |
| `03_the_linker_script.md` | `../04_the_linker_script.md` |
| `04_boot_and_vectors.md` | `../05_boot_and_vectors.md` |
| `05_reset_handler.md` | `../06_reset_handler.md` |
| `06_registers_and_bits.md` | `../07_registers_and_bits.md` |
| `07_gpio.md` | `../08_first_blink.md` and `../09_gpio_reference.md` |

Revision B additionally has `01_setup_and_workspace.md`, the toolchain and
workspace chapter revision A never had.
