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

    /// Return whether this model belongs to the original ILUMA family.
    #[must_use]
    pub const fn is_iluma_family(self) -> bool {
        matches!(self, Self::IlumaOne | Self::Iluma | Self::IlumaPrime)
    }

    /// Return whether this model belongs to the ILUMA i family.
    #[must_use]
    pub const fn is_iluma_i_family(self) -> bool {
        matches!(self, Self::IlumaIOne | Self::IlumaI | Self::IlumaIPrime)
    }

    /// Return whether this model exposes holder-specific metadata and settings.
    ///
    /// Observed behavior: `iqos_cli` gates holder-scoped settings through
    /// `is_iluma_or_higher()`, which currently only includes `Iluma` and
    /// `IlumaI`.
    #[must_use]
    pub const fn supports_holder_features(self) -> bool {
        matches!(self, Self::Iluma | Self::IlumaI)
    }

    /// Return whether this model supports the requested device capability.
    ///
    /// This matrix is derived from the currently implemented IQOS families in
    /// `iqos_cli` plus the observed split between one-piece and holder-based
    /// devices for holder-scoped settings.
    #[must_use]
    pub const fn supports(self, capability: DeviceCapability) -> bool {
        match capability {
            DeviceCapability::Brightness
            | DeviceCapability::FlexPuff
            | DeviceCapability::SmartGesture
            | DeviceCapability::AutoStart => self.supports_holder_features(),
            DeviceCapability::Vibration | DeviceCapability::DeviceLock => {
                !matches!(self, Self::Unknown)
            }
            DeviceCapability::FlexBattery => matches!(self, Self::IlumaI),
        }
    }

    /// Return whether this model supports the charge-start vibration setting.
    #[must_use]
    pub const fn supports_charge_start_vibration(self) -> bool {
        self.supports_holder_features()
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

#[cfg(test)]
mod tests {
    use super::{DeviceCapability, DeviceModel};

    #[test]
    fn detects_one_form_factor_models() {
        assert!(DeviceModel::IlumaOne.is_one_form_factor());
        assert!(DeviceModel::IlumaIOne.is_one_form_factor());
        assert!(!DeviceModel::Iluma.is_one_form_factor());
        assert!(!DeviceModel::Unknown.is_one_form_factor());
    }

    #[test]
    fn detects_model_families() {
        assert!(DeviceModel::IlumaPrime.is_iluma_family());
        assert!(DeviceModel::IlumaIPrime.is_iluma_i_family());
        assert!(!DeviceModel::IlumaIPrime.is_iluma_family());
        assert!(!DeviceModel::Unknown.is_iluma_i_family());
    }

    #[test]
    fn maps_capabilities_by_model_variation() {
        assert!(DeviceModel::Iluma.supports(DeviceCapability::Brightness));
        assert!(DeviceModel::IlumaI.supports(DeviceCapability::FlexPuff));
        assert!(DeviceModel::IlumaI.supports(DeviceCapability::SmartGesture));
        assert!(DeviceModel::IlumaI.supports(DeviceCapability::FlexBattery));
        assert!(!DeviceModel::IlumaOne.supports(DeviceCapability::Brightness));
        assert!(!DeviceModel::IlumaPrime.supports(DeviceCapability::FlexPuff));
        assert!(!DeviceModel::IlumaIPrime.supports(DeviceCapability::SmartGesture));
        assert!(!DeviceModel::IlumaIOne.supports(DeviceCapability::FlexBattery));
        assert!(!DeviceModel::IlumaPrime.supports(DeviceCapability::FlexBattery));
        assert!(!DeviceModel::Unknown.supports(DeviceCapability::Vibration));
    }

    #[test]
    fn gates_charge_start_vibration_to_holder_models() {
        assert!(DeviceModel::Iluma.supports_charge_start_vibration());
        assert!(DeviceModel::IlumaI.supports_charge_start_vibration());
        assert!(!DeviceModel::IlumaPrime.supports_charge_start_vibration());
        assert!(!DeviceModel::IlumaIPrime.supports_charge_start_vibration());
        assert!(!DeviceModel::IlumaIOne.supports_charge_start_vibration());
    }
}
