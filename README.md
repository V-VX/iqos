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

- Brightness, vibration, FlexPuff _(ILUMA i)_
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

## Local Hardware Debug

The repository includes a local BLE debug binary at [`debug/hardware_ble.rs`](/Users/vvx/projekt/rs/iqos/debug/hardware_ble.rs).

Basic run:

```bash
cargo run --features btleplug-support --bin hardware_ble
```

Run against a specific device name:

```bash
IQOS_TEST_NAME_SUBSTRING="ILUMA" \
  cargo run --features btleplug-support --bin hardware_ble
```

Enable setting changes and verification:

```bash
IQOS_TEST_ALLOW_STATEFUL_WRITES=1 \
  cargo run --features btleplug-support --bin hardware_ble
```

Useful environment variables:

- `IQOS_TEST_NAME_SUBSTRING` — optional device-name filter
- `IQOS_TEST_ALLOW_STATEFUL_WRITES` — enable write operations and verification loops (`1` or `true`)
- `IQOS_TEST_VIBRATE_MILLIS` — vibration duration for locate-device steps (default: `500`)

When `IQOS_TEST_ALLOW_STATEFUL_WRITES=1` is set, the binary reads the current setting, writes the opposite value, reads again to verify the change, restores the original value, and reads again to verify restoration for settings that support read-back (`brightness`, `FlexPuff`, `vibration`, `FlexBattery`).

`autostart` and `smartgesture` are also checked, but their status read is still experimental and based on a reverse-engineered probe in the debug binary rather than a stable library API.

Direct commands such as vibration bursts and lock/unlock do not currently have a matching read-back status in the library. Those steps are still executed, and the binary finishes them in a known end state: vibration stopped and device unlocked.

### Focused Feature Debug Binaries

Implementation-local real-device probes should live in a single file directly under `debug/`, not
inside the integrated `hardware_ble` harness. Register each file as its own binary target in
`Cargo.toml`.

Example:

```toml
[[bin]]
name = "autostart"
path = "debug/autostart.rs"
required-features = ["btleplug-support"]
```

Then run only that focused probe:

```bash
IQOS_TEST_ALLOW_STATEFUL_WRITES=1 \
  cargo run --features btleplug-support --bin autostart
```

---

## License

GPL-3.0 — see [LICENSE](LICENSE).
