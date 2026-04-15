use crate::{Error, Result};

use super::{DeviceCapability, DeviceModel};

/// Command used to request the main vibration settings frame.
pub const LOAD_VIBRATION_SETTINGS_COMMAND: [u8; 5] = [0x00, 0xC9, 0x00, 0x23, 0xE9];

/// Command used to request the charge-start vibration setting on holder-based models.
pub const LOAD_VIBRATE_CHARGE_START_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08];

const WHEN_HEATING_START_FLAG: u16 = 0x0100;
const WHEN_STARTING_TO_USE_FLAG: u16 = 0x1000;
const WHEN_MANUALLY_TERMINATED_FLAG: u16 = 0x0010;
const WHEN_PUFF_END_FLAG: u16 = 0x0001;

const WHEN_CHARGE_START_ON_RESPONSE: [u8; 19] = [
    0x00, 0x08, 0x8B, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x56,
];
const WHEN_CHARGE_START_OFF_RESPONSE: [u8; 19] = [
    0x00, 0x08, 0x8B, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xEE,
];

const WHEN_CHARGING_START_ON_COMMANDS: [&[u8]; 7] = [
    &[
        0x01, 0xC9, 0x4F, 0x04, 0x5B, 0x04, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06],
    &[
        0x01, 0xC9, 0x4F, 0x04, 0x72, 0x05, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x72],
    &[0x00, 0xC9, 0x47, 0x04, 0x00, 0xFF, 0xFF, 0x00, 0xDA],
    &[0x00, 0xC9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08],
    &[0x00, 0xC9, 0x07, 0x04, 0x05, 0x00, 0x00, 0x00, 0x1E],
];

const WHEN_CHARGING_START_OFF_COMMANDS: [&[u8]; 7] = [
    &[
        0x01, 0xC9, 0x4F, 0x04, 0x64, 0x04, 0x00, 0xFF, 0xFF, 0xFF, 0x09, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C],
    &[
        0x01, 0xC9, 0x4F, 0x04, 0x4D, 0x05, 0x00, 0xFF, 0xFF, 0xFF, 0x09, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78],
    &[0x00, 0xC9, 0x47, 0x04, 0x00, 0xFF, 0xFF, 0x00, 0xDA],
    &[0x00, 0xC9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08],
    &[0x00, 0xC9, 0x07, 0x04, 0x05, 0x00, 0x00, 0x00, 0x1E],
];

/// Typed vibration configuration snapshot.
///
/// `when_charging_start` is only available on holder-based models. One-piece
/// models leave it as `None`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VibrationSettings {
    charge_start: Option<bool>,
    heating_start: bool,
    starting_to_use: bool,
    puff_end: bool,
    manually_terminated: bool,
}

impl VibrationSettings {
    /// Create a vibration snapshot for models without charge-start support.
    #[allow(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub const fn new(
        when_heating_start: bool,
        when_starting_to_use: bool,
        when_puff_end: bool,
        when_manually_terminated: bool,
    ) -> Self {
        Self {
            charge_start: None,
            heating_start: when_heating_start,
            starting_to_use: when_starting_to_use,
            puff_end: when_puff_end,
            manually_terminated: when_manually_terminated,
        }
    }

    /// Create a vibration snapshot for holder-based models.
    #[allow(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub const fn with_charge_start(
        when_heating_start: bool,
        when_starting_to_use: bool,
        when_puff_end: bool,
        when_manually_terminated: bool,
        when_charging_start: bool,
    ) -> Self {
        Self {
            charge_start: Some(when_charging_start),
            heating_start: when_heating_start,
            starting_to_use: when_starting_to_use,
            puff_end: when_puff_end,
            manually_terminated: when_manually_terminated,
        }
    }

    /// Parse the main vibration response frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the response is too short or does not match the
    /// observed vibration response header.
    pub fn from_response(bytes: &[u8], model: DeviceModel) -> Result<Self> {
        if bytes.len() < 9 {
            return Err(Error::ProtocolDecode(
                "invalid vibration response: frame too short".to_string(),
            ));
        }

        if bytes[0] != 0x00 || bytes[1] != 0x08 || bytes[2] != 0x84 || bytes[3] != 0x23 {
            return Err(Error::ProtocolDecode(
                "invalid vibration response: header mismatch".to_string(),
            ));
        }

        if model.supports_charge_start_vibration() {
            if bytes[4] != 0x10 && bytes[4] != 0x03 {
                return Err(Error::ProtocolDecode(
                    "invalid vibration response: header mismatch".to_string(),
                ));
            }
        } else if bytes[4] != 0x10 {
            return Err(Error::ProtocolDecode(
                "invalid vibration response: header mismatch".to_string(),
            ));
        }

        let heat_use_byte = bytes[6];
        let end_terminated_byte = bytes[7];

        Ok(Self::new(
            (heat_use_byte & 0x01) != 0,
            (heat_use_byte & 0x10) != 0,
            (end_terminated_byte & 0x01) != 0,
            (end_terminated_byte & 0x10) != 0,
        ))
    }

    /// Parse the charge-start vibration response bit used by holder-based models.
    ///
    /// # Errors
    ///
    /// Returns an error if the response is too short.
    pub fn charge_start_from_response(bytes: &[u8]) -> Result<bool> {
        if bytes.len() < 19 {
            return Err(Error::ProtocolDecode(
                "invalid charge-start vibration response: frame too short".to_string(),
            ));
        }

        if bytes == WHEN_CHARGE_START_ON_RESPONSE {
            Ok(true)
        } else if bytes == WHEN_CHARGE_START_OFF_RESPONSE {
            Ok(false)
        } else {
            Ok(bytes[8] == 0x01)
        }
    }

    /// Return a copy of these settings with an explicit charge-start state attached.
    #[must_use]
    pub const fn with_observed_charge_start(mut self, when_charging_start: bool) -> Self {
        self.charge_start = Some(when_charging_start);
        self
    }

    /// Return whether vibration is enabled when charging starts.
    #[must_use]
    pub const fn when_charging_start(self) -> Option<bool> {
        self.charge_start
    }

    /// Return whether vibration is enabled when heating starts.
    #[must_use]
    pub const fn when_heating_start(self) -> bool {
        self.heating_start
    }

    /// Return whether vibration is enabled when a session starts.
    #[must_use]
    pub const fn when_starting_to_use(self) -> bool {
        self.starting_to_use
    }

    /// Return whether vibration is enabled near puff end.
    #[must_use]
    pub const fn when_puff_end(self) -> bool {
        self.puff_end
    }

    /// Return whether vibration is enabled on manual termination.
    #[must_use]
    pub const fn when_manually_terminated(self) -> bool {
        self.manually_terminated
    }

    /// Build the vibration update command sequence for the given model.
    ///
    /// # Errors
    ///
    /// Returns an error when the model does not support vibration updates or
    /// when charge-start data is provided for a model that does not expose it.
    pub fn build_update_commands(self, model: DeviceModel) -> Result<Vec<Vec<u8>>> {
        if !model.supports(DeviceCapability::Vibration) {
            return Err(Error::Unsupported(format!(
                "vibration is not supported for model {model:?}"
            )));
        }

        if !model.supports_charge_start_vibration() && self.charge_start.is_some() {
            return Err(Error::Unsupported(format!(
                "charge-start vibration is not supported for model {model:?}"
            )));
        }
        if model.supports_charge_start_vibration() && self.charge_start.is_none() {
            return Err(Error::ProtocolEncode(format!(
                "charge-start vibration value is required for model {model:?}"
            )));
        }

        let mut commands = vec![build_main_vibration_command(self)];

        if model.supports_charge_start_vibration() {
            let Some(charge_start_enabled) = self.charge_start else {
                return Err(Error::ProtocolEncode(format!(
                    "charge-start vibration value is required for model {model:?}"
                )));
            };
            let extra_commands = if charge_start_enabled {
                &WHEN_CHARGING_START_ON_COMMANDS
            } else {
                &WHEN_CHARGING_START_OFF_COMMANDS
            };

            if charge_start_enabled || !all_core_settings_disabled(self) {
                commands.extend(extra_commands.iter().map(|command| command.to_vec()));
            }
        }

        Ok(commands)
    }
}

fn all_core_settings_disabled(settings: VibrationSettings) -> bool {
    !settings.heating_start
        && !settings.starting_to_use
        && !settings.puff_end
        && !settings.manually_terminated
}

fn build_main_vibration_command(settings: VibrationSettings) -> Vec<u8> {
    let mut register = 0_u16;

    if settings.heating_start {
        register |= WHEN_HEATING_START_FLAG;
    }
    if settings.starting_to_use {
        register |= WHEN_STARTING_TO_USE_FLAG;
    }
    if settings.puff_end {
        register |= WHEN_PUFF_END_FLAG;
    }
    if settings.manually_terminated {
        register |= WHEN_MANUALLY_TERMINATED_FLAG;
    }

    let mut command = vec![0x00, 0xC9, 0x44, 0x23, 0x10, 0x00];
    command.push((register >> 8) as u8);
    command.push((register & 0xFF) as u8);
    command.push(vibration_checksum(register));
    command
}

fn vibration_checksum(register: u16) -> u8 {
    let mut checksum = 0x77;

    if (register & WHEN_PUFF_END_FLAG) != 0 {
        checksum ^= 0x07;
    }
    if (register & WHEN_MANUALLY_TERMINATED_FLAG) != 0 {
        checksum ^= 0x70;
    }
    if (register & WHEN_HEATING_START_FLAG) != 0 {
        checksum ^= 0x15;
    }
    if (register & WHEN_STARTING_TO_USE_FLAG) != 0 {
        checksum ^= 0x57;
    }

    checksum
}

#[cfg(test)]
mod tests {
    use super::{
        LOAD_VIBRATE_CHARGE_START_COMMAND, LOAD_VIBRATION_SETTINGS_COMMAND, VibrationSettings,
        WHEN_CHARGING_START_ON_COMMANDS,
    };
    use crate::protocol::DeviceModel;

    #[test]
    fn parses_basic_vibration_response() {
        let settings = VibrationSettings::from_response(
            &[0x00, 0x08, 0x84, 0x23, 0x10, 0x00, 0x01, 0x10, 0x77],
            DeviceModel::IlumaOne,
        )
        .unwrap();

        assert_eq!(settings, VibrationSettings::new(true, false, false, true));
    }

    #[test]
    fn parses_iluma_alt_header_vibration_response() {
        let settings = VibrationSettings::from_response(
            &[0x00, 0x08, 0x84, 0x23, 0x03, 0x00, 0x10, 0x01, 0x77],
            DeviceModel::Iluma,
        )
        .unwrap();

        assert_eq!(settings, VibrationSettings::new(false, true, true, false));
    }

    #[test]
    fn rejects_iluma_alt_header_vibration_response_for_one_piece_model() {
        let error = VibrationSettings::from_response(
            &[0x00, 0x08, 0x84, 0x23, 0x03, 0x00, 0x10, 0x01, 0x77],
            DeviceModel::IlumaOne,
        )
        .unwrap_err();

        assert!(
            matches!(error, crate::Error::ProtocolDecode(message) if message.contains("header mismatch"))
        );
    }

    #[test]
    fn parses_charge_start_response_variants() {
        assert!(
            VibrationSettings::charge_start_from_response(&[
                0x00, 0x08, 0x8B, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x56,
            ])
            .unwrap()
        );

        assert!(
            !VibrationSettings::charge_start_from_response(&[
                0x00, 0x08, 0x8B, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0xEE,
            ])
            .unwrap()
        );
    }

    #[test]
    fn keeps_load_commands_stable() {
        assert_eq!(LOAD_VIBRATION_SETTINGS_COMMAND, [0x00, 0xC9, 0x00, 0x23, 0xE9]);
        assert_eq!(
            LOAD_VIBRATE_CHARGE_START_COMMAND,
            [0x00, 0xC9, 0x07, 0x04, 0x04, 0x00, 0x00, 0x00, 0x08]
        );
    }

    #[test]
    fn builds_basic_vibration_update_command() {
        let commands = VibrationSettings::new(true, false, true, false)
            .build_update_commands(DeviceModel::IlumaOne)
            .unwrap();

        assert_eq!(commands, vec![vec![0x00, 0xC9, 0x44, 0x23, 0x10, 0x00, 0x01, 0x01, 0x65]]);
    }

    #[test]
    fn builds_iluma_charge_start_sequence() {
        let commands = VibrationSettings::with_charge_start(true, false, false, false, true)
            .build_update_commands(DeviceModel::Iluma)
            .unwrap();

        let expected: Vec<Vec<u8>> =
            std::iter::once(vec![0x00, 0xC9, 0x44, 0x23, 0x10, 0x00, 0x01, 0x00, 0x62])
                .chain(WHEN_CHARGING_START_ON_COMMANDS.iter().map(|command| command.to_vec()))
                .collect();

        assert_eq!(commands, expected);
    }

    #[test]
    fn rejects_charge_start_on_one_piece_model() {
        let error = VibrationSettings::with_charge_start(false, false, false, false, true)
            .build_update_commands(DeviceModel::IlumaOne)
            .unwrap_err();

        assert!(
            matches!(error, crate::Error::Unsupported(message) if message.contains("charge-start vibration"))
        );
    }

    #[test]
    fn rejects_holder_update_without_charge_start_value() {
        let error = VibrationSettings::new(false, false, false, false)
            .build_update_commands(DeviceModel::Iluma)
            .unwrap_err();

        assert!(
            matches!(error, crate::Error::ProtocolEncode(message) if message.contains("required"))
        );
    }
}
