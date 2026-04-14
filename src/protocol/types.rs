/// Known IQOS device models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceModel {
    /// IQOS ILUMA generation.
    Iluma,
    /// IQOS ILUMA i generation.
    IlumaI,
    /// IQOS ILUMA ONE form factor.
    IlumaOne,
    /// Unknown or not-yet-modeled device family.
    Unknown,
}

/// Device-level feature capability flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceCapability {
    /// Brightness control support.
    Brightness,
    /// Vibration configuration support.
    Vibration,
    /// `FlexPuff` support.
    FlexPuff,
    /// `FlexBattery` support.
    FlexBattery,
    /// Smart gesture support.
    SmartGesture,
    /// Auto-start support.
    AutoStart,
    /// Lock/unlock support.
    DeviceLock,
}
