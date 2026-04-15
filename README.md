# IQOS

Rust library for controlling IQOS devices over BLE, exposing device internals not accessible through the official IQOS app.

## What This Exposes

The official IQOS app provides basic status and settings. This library goes further by surfacing **diagnostic telemetry that the app does not expose**:

- Total puff (smoking) count — lifetime usage counter
- Days used — how long the device has been in service
- Battery voltage — raw cell voltage, not just a percentage

Beyond telemetry, the library also provides programmatic control over settings the app either hides or makes cumbersome:

- Brightness, vibration, FlexPuff, FlexBattery, Pause Mode
- Smart Gesture and Auto Start
- Device lock / unlock

## Status

Early development. The public API is still taking shape and should be considered unstable.

**Transport support:**
- BLE (Bluetooth Low Energy) — implemented, enabled via the `btleplug-support` feature
- USB — not yet implemented; the architecture is designed to support it, but no USB backend exists yet

## Architecture

```text
src/
├── lib.rs              # Public facade — Iqos<T> device handle
├── error.rs            # Error types and Result alias
├── transport.rs        # Transport trait shared by BLE/USB backends
├── protocol/           # Command builders, response parsers, typed domain values
│   ├── ble.rs
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
    └── usb.rs          # USB stub (usb-support feature, not yet implemented)
```

## Features

- `btleplug-support` — enables BLE backend via [`btleplug`](https://github.com/deviceplug/btleplug)
- `usb-support` — reserved for future USB transport (not yet implemented)

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

## License

GPL-3.0 — see [LICENSE](LICENSE).
