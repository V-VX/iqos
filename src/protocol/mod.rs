//! Protocol-layer types and scaffolding.
//!
//! This module will hold extracted command builders, response parsers, protocol
//! constants, and typed domain models derived from the working `iqos_cli`
//! implementation.

mod ble;
mod brightness;
mod commands;
mod constants;
mod firmware;
mod responses;
mod types;

pub use ble::{
    BATTERY_CHARACTERISTIC_UUID, DEVICE_INFO_SERVICE_UUID, HOLDER_PRODUCT_NUMBER_COMMAND,
    IQOS_CORE_SERVICE_UUID, MANUFACTURER_NAME_CHAR_UUID_PREFIX, MODEL_NUMBER_CHAR_UUID_PREFIX,
    PRODUCT_NUMBER_COMMAND, SCP_CONTROL_CHARACTERISTIC_UUID, SERIAL_NUMBER_CHAR_UUID_PREFIX,
    SOFTWARE_REVISION_CHAR_UUID_PREFIX,
};
pub use brightness::{
    BrightnessLevel, LOAD_BRIGHTNESS_COMMAND, SET_BRIGHTNESS_HIGH_COMMANDS,
    SET_BRIGHTNESS_LOW_COMMANDS,
};
pub use commands::CommandFrame;
pub use constants::IQOS_PROTOCOL_FAMILY;
pub use firmware::{
    FirmwareKind, FirmwareVersion, LOAD_HOLDER_FIRMWARE_VERSION_COMMAND,
    LOAD_STICK_FIRMWARE_VERSION_COMMAND,
};
pub use responses::ResponseFrame;
pub use types::{DeviceCapability, DeviceInfo, DeviceModel};
