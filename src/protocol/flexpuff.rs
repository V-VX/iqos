use crate::{Error, Result};

/// Command used to request the current `FlexPuff` setting.
pub const LOAD_FLEXPUFF_COMMAND: [u8; 9] = [0x00, 0xD2, 0x05, 0x22, 0x03, 0x00, 0x00, 0x00, 0x17];

/// Command used to enable `FlexPuff`.
pub(crate) const FLEXPUFF_ENABLE_COMMAND: [u8; 9] =
    [0x00, 0xD2, 0x45, 0x22, 0x03, 0x01, 0x00, 0x00, 0x0A];

/// Command used to disable `FlexPuff`.
pub(crate) const FLEXPUFF_DISABLE_COMMAND: [u8; 9] =
    [0x00, 0xD2, 0x45, 0x22, 0x03, 0x00, 0x00, 0x00, 0x0A];

/// `FlexPuff` enabled/disabled setting.
///
/// `FlexPuff` is a feature available on IQOS ILUMA i and ILUMA i PRIME that
/// adjusts puff resistance feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlexPuffSetting {
    enabled: bool,
}

impl FlexPuffSetting {
    /// Create a `FlexPuffSetting` with the given enabled state.
    #[must_use]
    pub const fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Return whether `FlexPuff` is enabled.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    /// Parse a `FlexPuffSetting` from a protocol response frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the frame is too short, the header bytes do not
    /// match the observed `FlexPuff` response format, or the flag byte is
    /// unknown.
    pub fn from_response(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 9 {
            return Err(Error::ProtocolDecode(
                "invalid FlexPuff response: frame too short".to_string(),
            ));
        }

        if bytes[0] != 0x00
            || bytes[1] != 0x90
            || bytes[2] != 0x85
            || bytes[3] != 0x22
            || bytes[4] != 0x03
        {
            return Err(Error::ProtocolDecode(
                "invalid FlexPuff response: header mismatch".to_string(),
            ));
        }

        match bytes[5] {
            0x01 => Ok(Self { enabled: true }),
            0x00 => Ok(Self { enabled: false }),
            _ => Err(Error::ProtocolDecode(
                "invalid FlexPuff response: unknown flag byte".to_string(),
            )),
        }
    }

    /// Return the write command for this setting.
    #[must_use]
    pub const fn write_command(self) -> &'static [u8] {
        if self.enabled { &FLEXPUFF_ENABLE_COMMAND } else { &FLEXPUFF_DISABLE_COMMAND }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FLEXPUFF_DISABLE_COMMAND, FLEXPUFF_ENABLE_COMMAND, FlexPuffSetting, LOAD_FLEXPUFF_COMMAND,
    };

    #[test]
    fn parses_enabled_flexpuff_response() {
        let setting =
            FlexPuffSetting::from_response(&[0x00, 0x90, 0x85, 0x22, 0x03, 0x01, 0x00, 0x00, 0x00]);
        assert_eq!(setting.unwrap(), FlexPuffSetting::new(true));
    }

    #[test]
    fn parses_disabled_flexpuff_response() {
        let setting =
            FlexPuffSetting::from_response(&[0x00, 0x90, 0x85, 0x22, 0x03, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(setting.unwrap(), FlexPuffSetting::new(false));
    }

    #[test]
    fn rejects_short_flexpuff_response() {
        let error = FlexPuffSetting::from_response(&[0x00, 0x90, 0x85, 0x22]);
        assert!(error.is_err());
    }

    #[test]
    fn rejects_invalid_flexpuff_header() {
        let error =
            FlexPuffSetting::from_response(&[0x00, 0x00, 0x85, 0x22, 0x03, 0x01, 0x00, 0x00, 0x00]);
        assert!(error.is_err());
    }

    #[test]
    fn rejects_unknown_flag_byte() {
        let error =
            FlexPuffSetting::from_response(&[0x00, 0x90, 0x85, 0x22, 0x03, 0xFF, 0x00, 0x00, 0x00]);
        assert!(error.is_err());
    }

    #[test]
    fn write_command_selects_correct_constant() {
        assert_eq!(FlexPuffSetting::new(true).write_command(), &FLEXPUFF_ENABLE_COMMAND);
        assert_eq!(FlexPuffSetting::new(false).write_command(), &FLEXPUFF_DISABLE_COMMAND);
    }

    #[test]
    fn keeps_load_command_stable() {
        assert_eq!(LOAD_FLEXPUFF_COMMAND, [0x00, 0xD2, 0x05, 0x22, 0x03, 0x00, 0x00, 0x00, 0x17]);
    }
}
