use crate::protocol::FirmwareVersion;

/// Aggregated firmware and battery snapshot for a connected IQOS device.
///
/// For holder models (`Iluma`, `IlumaPrime`, `IlumaI`, `IlumaIPrime`) both
/// `stick_firmware` and `holder_firmware` are populated. For one-piece models
/// `holder_firmware` is `None`. `battery_voltage` is `None` when the
/// diagnostic read fails.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceStatus {
    /// Firmware version reported by the stick (or the device itself on one-piece models).
    pub stick_firmware: FirmwareVersion,
    /// Firmware version reported by the holder — `Some` only for folder-type models.
    pub holder_firmware: Option<FirmwareVersion>,
    /// Battery cell voltage in volts (e.g. `4.2`), or `None` if the read failed.
    pub battery_voltage: Option<f32>,
}
