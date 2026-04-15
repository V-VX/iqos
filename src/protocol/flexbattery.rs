use crate::{Error, Result};

/// Command used to request the current `FlexBattery` mode.
pub const LOAD_FLEXBATTERY_COMMAND: [u8; 5] = [0x00, 0xC9, 0x00, 0x25, 0xFB];

/// Command used to request the current Pause Mode setting.
pub const LOAD_PAUSEMODE_COMMAND: [u8; 9] = [0x00, 0xC9, 0x07, 0x24, 0x02, 0x00, 0x00, 0x00, 0x18];

/// First command in the `FlexBattery` Eco mode set sequence.
pub(crate) const FLEXBATTERY_ECO_SET_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x44, 0x25, 0x01, 0x00, 0x00, 0x00, 0x4D];

/// First command in the `FlexBattery` Performance mode set sequence.
pub(crate) const FLEXBATTERY_PERFORMANCE_SET_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x44, 0x25, 0x00, 0x00, 0x00, 0x00, 0x5B];

/// First command in the Pause Mode enable set sequence.
pub(crate) const PAUSEMODE_ENABLE_SET_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x47, 0x24, 0x02, 0x01, 0x00, 0x00, 0x05];

/// First command in the Pause Mode disable set sequence.
pub(crate) const PAUSEMODE_DISABLE_SET_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x47, 0x24, 0x02, 0x00, 0x00, 0x00, 0x6E];

/// `FlexBattery` operating mode.
///
/// `FlexBattery` is supported on IQOS ILUMA i holder-based models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexBatteryMode {
    /// Standard Performance mode (default).
    #[default]
    Performance,
    /// Eco mode — extends battery life.
    Eco,
}

impl FlexBatteryMode {
    /// Parse a `FlexBatteryMode` from a protocol response frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the frame is too short, the header bytes do not
    /// match the observed `FlexBattery` response format, or the flag byte is
    /// unknown.
    pub fn from_response(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 9 {
            return Err(Error::ProtocolDecode(
                "invalid FlexBattery response: frame too short".to_string(),
            ));
        }

        if bytes[0] != 0x00 || bytes[1] != 0x08 || bytes[2] != 0x84 || bytes[3] != 0x25 {
            return Err(Error::ProtocolDecode(
                "invalid FlexBattery response: header mismatch".to_string(),
            ));
        }

        match bytes[4] {
            0x00 => Ok(Self::Performance),
            0x01 => Ok(Self::Eco),
            _ => Err(Error::ProtocolDecode(
                "invalid FlexBattery response: unknown mode byte".to_string(),
            )),
        }
    }

    /// Return the write command for this mode.
    ///
    /// This is the first command of the two-step write sequence; the caller
    /// must follow it with [`LOAD_FLEXBATTERY_COMMAND`] to complete the
    /// sequence.
    #[must_use]
    pub const fn write_command(self) -> &'static [u8] {
        match self {
            Self::Performance => &FLEXBATTERY_PERFORMANCE_SET_COMMAND,
            Self::Eco => &FLEXBATTERY_ECO_SET_COMMAND,
        }
    }
}

/// `FlexBattery` settings including operating mode and optional Pause Mode flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlexBatterySettings {
    mode: FlexBatteryMode,
    pause_mode: Option<bool>,
}

impl FlexBatterySettings {
    /// Create `FlexBatterySettings` with the given mode and optional Pause Mode state.
    #[must_use]
    pub const fn new(mode: FlexBatteryMode, pause_mode: Option<bool>) -> Self {
        Self { mode, pause_mode }
    }

    /// Return the `FlexBattery` operating mode.
    #[must_use]
    pub const fn mode(self) -> FlexBatteryMode {
        self.mode
    }

    /// Return the Pause Mode state if available.
    #[must_use]
    pub const fn pause_mode(self) -> Option<bool> {
        self.pause_mode
    }

    /// Parse `FlexBatterySettings` from a mode response frame and a Pause Mode
    /// response frame.
    ///
    /// # Errors
    ///
    /// Returns an error if either frame cannot be decoded.
    pub fn from_responses(mode_bytes: &[u8], pause_bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            mode: FlexBatteryMode::from_response(mode_bytes)?,
            pause_mode: Some(pausemode_from_response(pause_bytes)?),
        })
    }
}

/// Parse a Pause Mode state from a protocol response frame.
///
/// # Errors
///
/// Returns an error if the frame is too short, the header bytes do not match
/// the observed Pause Mode response format, or the flag byte is unknown.
pub fn pausemode_from_response(bytes: &[u8]) -> Result<bool> {
    if bytes.len() < 9 {
        return Err(Error::ProtocolDecode(
            "invalid Pause Mode response: frame too short".to_string(),
        ));
    }

    if bytes[0] != 0x00 || bytes[1] != 0x08 || bytes[2] != 0x87 || bytes[3] != 0x24 {
        return Err(Error::ProtocolDecode(
            "invalid Pause Mode response: header mismatch".to_string(),
        ));
    }

    match bytes[5] {
        0x00 => Ok(false),
        0x01 => Ok(true),
        _ => {
            Err(Error::ProtocolDecode("invalid Pause Mode response: unknown flag byte".to_string()))
        }
    }
}

/// Return the Pause Mode write command for the requested state.
///
/// This is the first command of the two-step write sequence; the caller
/// must follow it with [`LOAD_PAUSEMODE_COMMAND`] to complete the sequence.
#[must_use]
pub const fn pausemode_command(enabled: bool) -> &'static [u8] {
    if enabled { &PAUSEMODE_ENABLE_SET_COMMAND } else { &PAUSEMODE_DISABLE_SET_COMMAND }
}

#[cfg(test)]
mod tests {
    use super::{
        FLEXBATTERY_ECO_SET_COMMAND, FLEXBATTERY_PERFORMANCE_SET_COMMAND, FlexBatteryMode,
        FlexBatterySettings, LOAD_FLEXBATTERY_COMMAND, LOAD_PAUSEMODE_COMMAND,
        PAUSEMODE_DISABLE_SET_COMMAND, PAUSEMODE_ENABLE_SET_COMMAND, pausemode_command,
        pausemode_from_response,
    };

    #[test]
    fn parses_performance_flexbattery_response() {
        let mode =
            FlexBatteryMode::from_response(&[0x00, 0x08, 0x84, 0x25, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(mode.unwrap(), FlexBatteryMode::Performance);
    }

    #[test]
    fn parses_eco_flexbattery_response() {
        let mode =
            FlexBatteryMode::from_response(&[0x00, 0x08, 0x84, 0x25, 0x01, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(mode.unwrap(), FlexBatteryMode::Eco);
    }

    #[test]
    fn rejects_short_flexbattery_response() {
        assert!(FlexBatteryMode::from_response(&[0x00, 0x08, 0x84]).is_err());
    }

    #[test]
    fn rejects_invalid_flexbattery_header() {
        let error =
            FlexBatteryMode::from_response(&[0x00, 0x00, 0x84, 0x25, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert!(error.is_err());
    }

    #[test]
    fn rejects_unknown_flexbattery_mode_byte() {
        let error =
            FlexBatteryMode::from_response(&[0x00, 0x08, 0x84, 0x25, 0xFF, 0x00, 0x00, 0x00, 0x00]);
        assert!(error.is_err());
    }

    #[test]
    fn write_command_selects_correct_constant() {
        assert_eq!(
            FlexBatteryMode::Performance.write_command(),
            &FLEXBATTERY_PERFORMANCE_SET_COMMAND
        );
        assert_eq!(FlexBatteryMode::Eco.write_command(), &FLEXBATTERY_ECO_SET_COMMAND);
    }

    #[test]
    fn parses_pausemode_enabled_response() {
        let enabled =
            pausemode_from_response(&[0x00, 0x08, 0x87, 0x24, 0x02, 0x01, 0x00, 0x00, 0x00]);
        assert!(enabled.unwrap());
    }

    #[test]
    fn parses_pausemode_disabled_response() {
        let disabled =
            pausemode_from_response(&[0x00, 0x08, 0x87, 0x24, 0x02, 0x00, 0x00, 0x00, 0x00]);
        assert!(!disabled.unwrap());
    }

    #[test]
    fn rejects_short_pausemode_response() {
        assert!(pausemode_from_response(&[0x00, 0x08, 0x87]).is_err());
    }

    #[test]
    fn rejects_invalid_pausemode_header() {
        let error =
            pausemode_from_response(&[0x00, 0x00, 0x87, 0x24, 0x02, 0x01, 0x00, 0x00, 0x00]);
        assert!(error.is_err());
    }

    #[test]
    fn rejects_unknown_pausemode_flag() {
        let error =
            pausemode_from_response(&[0x00, 0x08, 0x87, 0x24, 0x02, 0xFF, 0x00, 0x00, 0x00]);
        assert!(error.is_err());
    }

    #[test]
    fn pausemode_command_selects_correct_constant() {
        assert_eq!(pausemode_command(true), &PAUSEMODE_ENABLE_SET_COMMAND);
        assert_eq!(pausemode_command(false), &PAUSEMODE_DISABLE_SET_COMMAND);
    }

    #[test]
    fn from_responses_combines_mode_and_pausemode() {
        let mode_bytes = [0x00, 0x08, 0x84, 0x25, 0x01, 0x00, 0x00, 0x00, 0x00];
        let pause_bytes = [0x00, 0x08, 0x87, 0x24, 0x02, 0x01, 0x00, 0x00, 0x00];
        let settings = FlexBatterySettings::from_responses(&mode_bytes, &pause_bytes).unwrap();
        assert_eq!(settings, FlexBatterySettings::new(FlexBatteryMode::Eco, Some(true)));
    }

    #[test]
    fn keeps_load_commands_stable() {
        assert_eq!(LOAD_FLEXBATTERY_COMMAND, [0x00, 0xC9, 0x00, 0x25, 0xFB]);
        assert_eq!(LOAD_PAUSEMODE_COMMAND, [0x00, 0xC9, 0x07, 0x24, 0x02, 0x00, 0x00, 0x00, 0x18]);
    }

    #[test]
    fn keeps_set_commands_stable() {
        assert_eq!(
            FLEXBATTERY_ECO_SET_COMMAND,
            [0x00, 0xC9, 0x44, 0x25, 0x01, 0x00, 0x00, 0x00, 0x4D]
        );
        assert_eq!(
            FLEXBATTERY_PERFORMANCE_SET_COMMAND,
            [0x00, 0xC9, 0x44, 0x25, 0x00, 0x00, 0x00, 0x00, 0x5B]
        );
        assert_eq!(
            PAUSEMODE_ENABLE_SET_COMMAND,
            [0x00, 0xC9, 0x47, 0x24, 0x02, 0x01, 0x00, 0x00, 0x05]
        );
        assert_eq!(
            PAUSEMODE_DISABLE_SET_COMMAND,
            [0x00, 0xC9, 0x47, 0x24, 0x02, 0x00, 0x00, 0x00, 0x6E]
        );
    }
}
