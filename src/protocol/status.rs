use crate::protocol::FirmwareVersion;

/// Aggregated firmware and battery snapshot for a connected IQOS device.
///
/// For models where
/// [`DeviceModel::supports_holder_features`](crate::protocol::DeviceModel::supports_holder_features)
/// returns `true`, both `stick_firmware` and `holder_firmware` are populated.
/// For other models, `holder_firmware` is `None`. `battery_voltage` is `None`
/// when the diagnostic transport read fails.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceStatus {
    /// Firmware version reported by the stick, or by the device itself on models
    /// where
    /// [`DeviceModel::supports_holder_features`](crate::protocol::DeviceModel::supports_holder_features)
    /// returns `false`.
    pub stick_firmware: FirmwareVersion,
    /// Firmware version reported by the holder when
    /// [`DeviceModel::supports_holder_features`](crate::protocol::DeviceModel::supports_holder_features)
    /// returns `true`.
    pub holder_firmware: Option<FirmwareVersion>,
    /// Battery cell voltage in volts (e.g. `4.2`), or `None` if the transport
    /// read failed.
    pub battery_voltage: Option<f32>,
}
