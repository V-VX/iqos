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
    #[must_use]
    pub const fn supports_holder_features(self) -> bool {
        matches!(self, Self::Iluma | Self::IlumaPrime | Self::IlumaI | Self::IlumaIPrime)
    }

    /// Return whether this model supports the requested device capability.
    ///
    /// Capability matrix:
    /// - `Brightness`, `Vibration`, `DeviceLock` — all known models (non-Unknown).
    /// - `FlexPuff`, `FlexBattery` — ILUMA i and ILUMA i PRIME only.
    /// - `AutoStart` — ILUMA i series only.
    /// - `SmartGesture`, `ChargeStartVibration` — holder form-factor models.
    #[must_use]
    pub const fn supports(self, capability: DeviceCapability) -> bool {
        match capability {
            DeviceCapability::Brightness
            | DeviceCapability::Vibration
            | DeviceCapability::DeviceLock => !matches!(self, Self::Unknown),
            DeviceCapability::FlexPuff | DeviceCapability::FlexBattery => {
                matches!(self, Self::IlumaI | Self::IlumaIPrime)
            }
            DeviceCapability::AutoStart => self.is_iluma_i_family(),
            DeviceCapability::SmartGesture | DeviceCapability::ChargeStartVibration => {
                self.supports_holder_features()
            }
        }
    }

    /// Return whether this model supports the charge-start vibration setting.
    #[must_use]
    pub const fn supports_charge_start_vibration(self) -> bool {
        self.supports(DeviceCapability::ChargeStartVibration)
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
///
/// This enum is non-exhaustive so downstream code keeps a wildcard arm as
/// reverse-engineered capabilities evolve.
#[non_exhaustive]
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
    /// Holder vibration feedback when the stick begins charging.
    ChargeStartVibration,
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
    fn detects_holder_feature_models() {
        for model in [
            DeviceModel::Iluma,
            DeviceModel::IlumaPrime,
            DeviceModel::IlumaI,
            DeviceModel::IlumaIPrime,
        ] {
            assert!(model.supports_holder_features(), "{model:?} should support holder features");
        }

        for model in [DeviceModel::IlumaOne, DeviceModel::IlumaIOne, DeviceModel::Unknown] {
            assert!(
                !model.supports_holder_features(),
                "{model:?} should not support holder features"
            );
        }
    }

    #[test]
    fn maps_capabilities_by_model_variation() {
        let models = [
            DeviceModel::IlumaOne,
            DeviceModel::Iluma,
            DeviceModel::IlumaPrime,
            DeviceModel::IlumaIOne,
            DeviceModel::IlumaI,
            DeviceModel::IlumaIPrime,
            DeviceModel::Unknown,
        ];

        let matrix = [
            (DeviceCapability::Brightness, [true, true, true, true, true, true, false]),
            (DeviceCapability::Vibration, [true, true, true, true, true, true, false]),
            (DeviceCapability::DeviceLock, [true, true, true, true, true, true, false]),
            (DeviceCapability::FlexPuff, [false, false, false, false, true, true, false]),
            (DeviceCapability::FlexBattery, [false, false, false, false, true, true, false]),
            (DeviceCapability::SmartGesture, [false, true, true, false, true, true, false]),
            (DeviceCapability::AutoStart, [false, false, false, true, true, true, false]),
            (DeviceCapability::ChargeStartVibration, [false, true, true, false, true, true, false]),
        ];

        for (capability, expected_by_model) in matrix {
            for (model, expected) in models.into_iter().zip(expected_by_model) {
                assert_eq!(
                    model.supports(capability),
                    expected,
                    "{model:?} support for {capability:?}"
                );
            }
        }
    }

    #[test]
    fn gates_charge_start_vibration_to_holder_models() {
        for model in [
            DeviceModel::Iluma,
            DeviceModel::IlumaPrime,
            DeviceModel::IlumaI,
            DeviceModel::IlumaIPrime,
        ] {
            assert!(model.supports_charge_start_vibration());
            assert!(model.supports(DeviceCapability::ChargeStartVibration));
        }
        assert!(!DeviceModel::IlumaOne.supports_charge_start_vibration());
        assert!(!DeviceModel::IlumaIOne.supports_charge_start_vibration());
        assert!(!DeviceModel::Unknown.supports_charge_start_vibration());
        assert!(!DeviceModel::IlumaOne.supports(DeviceCapability::ChargeStartVibration));
        assert!(!DeviceModel::IlumaIOne.supports(DeviceCapability::ChargeStartVibration));
        assert!(!DeviceModel::Unknown.supports(DeviceCapability::ChargeStartVibration));
    }
}
