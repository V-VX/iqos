# IQOS

[![CI](https://github.com/V-VX/iqos/actions/workflows/ci.yml/badge.svg)](https://github.com/V-VX/iqos/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)

Rust library for controlling IQOS devices over BLE, exposing device internals not accessible through the official IQOS app.

> **Early development.** The public API is still taking shape and should be considered unstable.

---

## What This Exposes

The official IQOS app surfaces only basic status and settings. This library goes further.

### Diagnostic telemetry — not available in the official app

| Field | Description |
|---|---|
| Total puff count | Lifetime usage counter |
| Days used | How long the device has been in service |
| Battery voltage | Raw cell voltage, not just a percentage |

### Programmatic device control

- Brightness, vibration, FlexPuff
- FlexBattery mode and Pause Mode _(ILUMA i)_
- Smart Gesture and Auto Start
- Device lock / unlock
- Firmware version (stick and holder)

---

## Transport Support

| Transport | Status | Feature flag |
|---|---|---|
| BLE (Bluetooth Low Energy) | ✅ Implemented | `btleplug-support` |
| USB | ⚠️ Not yet implemented | `usb-support` _(reserved)_ |

> ⚠️ **USB is not implemented.** The architecture is designed to support it — the `Transport` trait is transport-agnostic — but no USB backend exists yet.

---

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

### Design principles

- **Library-first** — no stdout/stderr output, no `unwrap()`/`panic!()` in library code
- **Typed protocol models** — all device state is represented as typed Rust values, not raw bytes
- **Transport-agnostic** — BLE and USB share the same `Transport` trait
- **Testable without hardware** — full coverage via `MockTransport`

---

## Development

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Local Hardware Tests

The repository includes BLE hardware integration tests under [`tests/hardware_ble.rs`](/Users/vvx/projekt/rs/iqos/tests/hardware_ble.rs). They are ignored by default and intended for local execution only.

These tests are excluded from normal `cargo test` and CI runs. They only exist when you explicitly opt in with `RUSTFLAGS='--cfg iqos_hardware_tests'`.

Set a stable substring of the target device name, then run the ignored test binary with BLE support enabled:

```bash
export IQOS_TEST_NAME_SUBSTRING="ILUMA"
RUSTFLAGS="--cfg iqos_hardware_tests" \
  cargo test --features btleplug-support --test hardware_ble -- --ignored --nocapture
```

Optional environment variables:

- `IQOS_TEST_SCAN_SECONDS` — override BLE scan duration in seconds (default: `5`)
- `IQOS_TEST_VIBRATE_MILLIS` — vibration duration for locate-device steps in the full sequence test (default: `750`)

These tests are read-only. They connect to the device, load metadata, and exercise supported read operations such as firmware, vibration, diagnostics, and model-specific settings.

To run the full stateful acceptance sequence that exercises every supported public operation in order, explicitly opt in to writes:

```bash
export IQOS_TEST_NAME_SUBSTRING="ILUMA"
export IQOS_TEST_ALLOW_STATEFUL_WRITES=1
RUSTFLAGS="--cfg iqos_hardware_tests" \
  cargo test --features btleplug-support --test hardware_ble hardware_exercises_all_supported_functions_in_sequence -- --ignored --nocapture
```

The full sequence restores settings that have read-back support (`brightness`, `FlexPuff`, `vibration`, `FlexBattery`). Write-only features such as `Smart Gesture` and `Auto Start` are exercised with a `disable -> enable -> disable` cycle, and lock/vibration helpers finish in the unlocked/stopped state.

---

## License

GPL-3.0 — see [LICENSE](LICENSE).
