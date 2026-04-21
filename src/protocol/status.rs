use crate::protocol::{DeviceInfo, DeviceModel, FirmwareVersion};

/// Aggregated device snapshot combining GATT metadata and SCP firmware reads.
///
/// Populated by [`Iqos::read_device_status`](crate::Iqos::read_device_status).
/// For models where
/// [`DeviceModel::supports_holder_features`](crate::protocol::DeviceModel::supports_holder_features)
/// returns `true`, holder product number and firmware fields are `Some`. For
/// one-piece models they are `None`.
/// `battery_voltage` is `None` when the SCP transport read fails.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceStatus {
    /// Detected device model.
    pub model: DeviceModel,
    /// Standard BLE device-information fields read at connection time.
    pub device_info: DeviceInfo,
    /// Product number reported by the stick, or by the device itself on one-piece models.
    pub product_number: String,
    /// Firmware version reported by the stick, or by the device itself on one-piece models.
    pub stick_firmware: FirmwareVersion,
    /// Product number reported by the holder when
    /// [`DeviceModel::supports_holder_features`](crate::protocol::DeviceModel::supports_holder_features)
    /// returns `true`.
    pub holder_product_number: Option<String>,
    /// Firmware version reported by the holder when
    /// [`DeviceModel::supports_holder_features`](crate::protocol::DeviceModel::supports_holder_features)
    /// returns `true`.
    pub holder_firmware: Option<FirmwareVersion>,
    /// Battery cell voltage in volts (e.g. `4.2`), or `None` if the SCP transport read failed.
    pub battery_voltage: Option<f32>,
}
