# Application templates

One sub-template per supported board. Each generates a standalone,
ready-to-flash application that consumes RustOS as a git dependency and
carries the board's build environment (`.cargo/config.toml` with the
target triple, linker flags, and flash runner).

Requires [cargo-generate](https://github.com/cargo-generate/cargo-generate)
(`cargo install cargo-generate`).

```sh
# Pick a board interactively:
cargo generate --git https://github.com/robinonsay/rustos

# Or name one directly:
cargo generate --git https://github.com/robinonsay/rustos templates/pico2
```

Then, from the generated project:

```sh
cargo run --release   # builds and flashes via picotool
```

## Boards

| Template | Board | Target |
|---|---|---|
| `templates/pico2` | Raspberry Pi Pico 2 (RP2350) | `thumbv8m.main-none-eabihf` |

## Co-developing an app and RustOS

Generated apps track RustOS's default branch on GitHub. To build against a
local checkout instead, add a patch section to the app's `Cargo.toml`:

```toml
[patch."https://github.com/robinonsay/rustos"]
api   = { path = "../rustos/api" }
pico2 = { path = "../rustos/firmware/pico2" }
```
