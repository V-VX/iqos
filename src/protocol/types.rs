/// Known IQOS device models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceModel {
    /// IQOS ILUMA ONE.
    IlumaOne,
    /// IQOS ILUMA.
    Iluma,
    /// IQOS ILUMA PRIME.
    IlumaPrime,
    /// IQOS ILUMA i ONE.
    IlumaIOne,
    /// IQOS ILUMA i.
    IlumaI,
    /// IQOS ILUMA i PRIME.
    IlumaIPrime,
    /// Unknown or not-yet-modeled device family.
    Unknown,
}

impl DeviceModel {
    /// Infer a model from a BLE local-name string.
    #[must_use]
    pub fn from_local_name(value: &str) -> Self {
        let normalized = value.trim().to_ascii_uppercase();

        if normalized.contains("ILUMA I PRIME") {
            Self::IlumaIPrime
        } else if normalized.contains("ILUMA I ONE") {
            Self::IlumaIOne
        } else if normalized.contains("ILUMA I") {
            Self::IlumaI
        } else if normalized.contains("ILUMA PRIME") {
            Self::IlumaPrime
        } else if normalized.contains("ILUMA ONE") {
            Self::IlumaOne
        } else if normalized.contains("ILUMA") {
            Self::Iluma
        } else {
            Self::Unknown
        }
    }

    /// Return whether this model uses the one-piece form factor.
    #[must_use]
    pub const fn is_one_form_factor(self) -> bool {
        matches!(self, Self::IlumaOne | Self::IlumaIOne)
    }
}

/// Snapshot of standard BLE device-information fields.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Standard model-number string.
    pub model_number: Option<String>,
    /// Standard serial-number string.
    pub serial_number: Option<String>,
    /// Standard software-revision string.
    pub software_revision: Option<String>,
    /// Standard manufacturer-name string.
    pub manufacturer_name: Option<String>,
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
