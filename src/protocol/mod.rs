//! Protocol-layer types and scaffolding.
//!
//! This module will hold extracted command builders, response parsers, protocol
//! constants, and typed domain models derived from the working `iqos_cli`
//! implementation.

mod ble;
mod brightness;
mod diagnosis;
mod firmware;
mod flexbattery;
mod flexpuff;
mod gesture;
mod lock;
mod types;
mod vibration;

/// Protocol family label used in documentation and diagnostics.
pub const IQOS_PROTOCOL_FAMILY: &str = "iqos";

pub use ble::{
    BATTERY_CHARACTERISTIC_UUID, DEVICE_INFO_SERVICE_UUID, HOLDER_PRODUCT_NUMBER_COMMAND,
    IQOS_CORE_SERVICE_UUID, MANUFACTURER_NAME_CHAR_UUID_PREFIX, MODEL_NUMBER_CHAR_UUID_PREFIX,
    PRODUCT_NUMBER_COMMAND, SCP_CONTROL_CHARACTERISTIC_UUID, SERIAL_NUMBER_CHAR_UUID_PREFIX,
    SOFTWARE_REVISION_CHAR_UUID_PREFIX,
};
pub use brightness::{BrightnessLevel, LOAD_BRIGHTNESS_COMMAND};
pub(crate) use diagnosis::DiagnosticDataBuilder;
pub use diagnosis::{
    ALL_DIAGNOSIS_COMMANDS, DiagnosticData, LOAD_BATTERY_VOLTAGE_COMMAND, LOAD_TELEMETRY_COMMAND,
    LOAD_TIMESTAMP_COMMAND,
};
pub use firmware::{
    FirmwareKind, FirmwareVersion, LOAD_HOLDER_FIRMWARE_VERSION_COMMAND,
    LOAD_STICK_FIRMWARE_VERSION_COMMAND,
};
pub(crate) use flexbattery::pausemode_command;
pub use flexbattery::{
    FlexBatteryMode, FlexBatterySettings, LOAD_FLEXBATTERY_COMMAND, LOAD_PAUSEMODE_COMMAND,
};
pub use flexpuff::{FlexPuffSetting, LOAD_FLEXPUFF_COMMAND};
pub use gesture::{autostart_command, smartgesture_command};
pub use lock::{lock_commands, unlock_commands};
pub use types::{DeviceCapability, DeviceInfo, DeviceModel};
pub use vibration::{
    LOAD_VIBRATE_CHARGE_START_COMMAND, LOAD_VIBRATION_SETTINGS_COMMAND, VibrationSettings,
};
