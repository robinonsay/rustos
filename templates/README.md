# Application templates

One sub-template per supported board. Each generates a standalone,
ready-to-flash application that consumes RustOS as a git dependency and
carries the board's build environment (`.cargo/config.toml` with the
target triple, linker flags, and flash runner).

Requires [cargo-generate](https://github.com/cargo-generate/cargo-generate)
0.23.4 or later (`cargo install cargo-generate`).

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

`templates/app` is not a board: it generates an application module for an
existing crate (see [Application modules](#application-modules)).

A template's manifest is `Cargo.toml.liquid`, never `Cargo.toml`: cargo
reads every `Cargo.toml` in a git dependency's repository, so a template's
`name = "{{project-name}}"` would put an error in the build output of every
crate that depends on RustOS. cargo-generate removes the `.liquid` suffix
when it generates the app.

## Application modules

`templates/app` generates one application (an `api::app::App`) for an
existing crate, not a crate of its own. It produces a module directory with
the same layout every app has:

| File | Holds |
|---|---|
| `mod.rs` | What the app does, and its public names |
| `app.rs` | The app: its state, its `App` hooks and its tests |
| `table.rs` | Its iLoad: its schedule slot and every value it is configured with, checked at compile time |

Generate it in the directory that holds the crate's apps. Pass `--force`
so cargo-generate keeps a snake_case name instead of changing it to
kebab-case, which is not a valid module name:

```sh
cd flight/src/apps
cargo generate --path ../../../deps/rustos templates/app --name air_data --force \
    --define summary="Air data from the pitot tube"
```

That writes `air_data/{mod,app,table}.rs`, with `AirDataApp` and its
table, `AirDataTable`. Then:

1. Add `pub mod air_data;` to the apps module.
2. Implement `From<api::app::HealthTlm<Id>>` for the crate's message type,
   where `Id` is the crate's app-id type, and pick the app's `health_mid`.
3. Point `MINOR_FRAME_MS` and `MAJOR_FRAME_MS` in `table.rs` at the
   crate's own schedule constants.
4. In `main`, build it with its id, table, input pipe and bus, put it in
   the scheduler's app array and register `table.slot.durations()`.
5. Write its work in `handle` and `on_process`, and move every number it
   uses into its table.

The generated app depends only on `api`: its schedule slot is an
`api::iload::Slot`, checked at compile time with `api::iload::assert_ok`,
and its health is an `api::app::Health`.

## Co-developing an app and RustOS

Generated apps track RustOS's default branch on GitHub. To build against a
local checkout instead, add a patch section to the app's `Cargo.toml`:

```toml
[patch."https://github.com/robinonsay/rustos"]
api   = { path = "../rustos/api" }
pico2 = { path = "../rustos/firmware/pico2" }
```
