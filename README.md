# IQOS

Rust library for controlling IQOS devices over BLE, with a clean architectural path for future USB transport support.

## Status

Early development. The public API is still taking shape and should be considered unstable.

## Architecture

```text
src/
├── lib.rs              # Public facade — Iqos<T> device handle
├── error.rs            # Error types and Result alias
├── transport.rs        # Transport trait shared by backends
├── protocol/           # Command builders, response parsers, typed domain values
│   ├── brightness.rs
│   ├── diagnosis.rs
│   ├── firmware.rs
│   ├── flexbattery.rs
│   ├── flexpuff.rs
│   ├── gesture.rs
│   ├── lock.rs
│   ├── types.rs
│   └── vibration.rs
└── transports/
    ├── ble_btleplug.rs # BLE backend (btleplug-support feature)
    └── usb.rs          # USB stub (usb-support feature, reserved)
```

## Features

- `btleplug-support` — enables BLE backend via [`btleplug`](https://github.com/deviceplug/btleplug)
- `usb-support` — reserved for future USB transport

## Supported Operations

- Firmware version (stick and holder)
- Brightness level (read / set)
- Vibration settings (read / set, holder and one-piece models)
- FlexPuff (read / set)
- FlexBattery mode and Pause Mode (read / set, ILUMA i)
- Smart Gesture and Auto Start (set)
- Device lock / unlock
- Diagnostic telemetry (puff count, days used, battery voltage)

## Design Principles

- Library-first: no stdout/stderr output, no `unwrap()`/`panic!()` in library code
- Typed protocol models: all device state is represented as typed Rust values, not raw bytes
- Transport-agnostic: BLE and USB are interchangeable backends behind the `Transport` trait
- Testable without hardware: full test coverage via `MockTransport`

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Debug CLI

The crate ships a small read-only BLE debug CLI behind the `btleplug-support` feature, intended for developer diagnostics.

```bash
cargo run --features btleplug-support -- inspect
cargo run --features btleplug-support -- inspect --name "prime"
cargo run --features btleplug-support -- probe brightness
cargo run --features btleplug-support -- probe firmware-stick
cargo run --features btleplug-support -- probe firmware-holder
cargo run --features btleplug-support -- probe battery
```

## License

GPL-3.0 — see [LICENSE](LICENSE).
