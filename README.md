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

## Local Hardware Debug Harness

The repository includes a local BLE debug harness at [`debug/hardware_ble.rs`](/Users/vvx/projekt/rs/iqos/debug/hardware_ble.rs). It is a developer-only binary and is not used by CI.

Set a stable substring of the target device name, then run the harness with BLE support enabled:

```bash
export IQOS_TEST_NAME_SUBSTRING="ILUMA"
cargo run --features btleplug-support --bin hardware_ble
```

Optional environment variables:

- `IQOS_TEST_ALLOW_STATEFUL_WRITES` — enable stateful write exercises (`1` or `true`)
- `IQOS_TEST_VIBRATE_MILLIS` — vibration duration for locate-device steps (default: `500`)

With `IQOS_TEST_ALLOW_STATEFUL_WRITES=1`, the harness runs command-by-command verification loops for the settings that have stable read-back support in the library (`brightness`, `FlexPuff`, `vibration`, `FlexBattery`):

- read current status
- send the opposite setting
- read status again and verify the change
- send the original setting
- read status again and verify restoration

Example:

```bash
export IQOS_TEST_NAME_SUBSTRING="ILUMA"
export IQOS_TEST_ALLOW_STATEFUL_WRITES=1
cargo run --features btleplug-support --bin hardware_ble
```

For `autostart` and `smartgesture`, the harness currently uses an experimental reverse-engineered status-probe path in the debug binary to discover a matching `0x00 C9 07 24 <subtype> 00 00 00 XX` read command before it performs the same verification loop. This is intentionally kept out of the library API until the protocol is confirmed more rigorously.

For direct commands such as vibration bursts and lock/unlock, the harness records the requests but cannot perform a true state read-back because the protocol does not currently expose a matching status read through the library. The harness always finishes those write-only steps in a known end state: vibration is stopped, and the device is left unlocked.

---

## License

GPL-3.0 — see [LICENSE](LICENSE).
