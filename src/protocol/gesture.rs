use crate::{Error, Result};

/// Command used to enable Smart Gesture.
///
/// Smart Gesture is supported on IQOS ILUMA and ILUMA i holder-based models.
pub(crate) const SMARTGESTURE_ENABLE_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x47, 0x24, 0x04, 0x01, 0x00, 0x00, 0x3C];

/// Command used to disable Smart Gesture.
pub(crate) const SMARTGESTURE_DISABLE_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x47, 0x24, 0x04, 0x00, 0x00, 0x00, 0x57];

/// Command used to enable Auto Start.
///
/// Auto Start is supported on IQOS ILUMA and ILUMA i holder-based models.
pub(crate) const AUTOSTART_ENABLE_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x47, 0x24, 0x01, 0x01, 0x00, 0x00, 0x3F];

/// Command used to disable Auto Start.
pub(crate) const AUTOSTART_DISABLE_COMMAND: [u8; 9] =
    [0x00, 0xC9, 0x47, 0x24, 0x01, 0x00, 0x00, 0x00, 0x54];

/// Return the Smart Gesture write command for the requested enabled state.
#[must_use]
pub const fn smartgesture_command(enabled: bool) -> &'static [u8] {
    if enabled { &SMARTGESTURE_ENABLE_COMMAND } else { &SMARTGESTURE_DISABLE_COMMAND }
}

/// Return the Auto Start write command for the requested enabled state.
#[must_use]
pub const fn autostart_command(enabled: bool) -> &'static [u8] {
    if enabled { &AUTOSTART_ENABLE_COMMAND } else { &AUTOSTART_DISABLE_COMMAND }
}

/// Command used to request the current Auto Start setting.
///
/// Experimental: derived from packet capture, not verified on hardware.
pub const LOAD_AUTOSTART_COMMAND: [u8; 9] = [0x00, 0xC9, 0x07, 0x24, 0x01, 0x00, 0x00, 0x00, 0x22];

/// Parse Auto Start state from a protocol response frame.
///
/// # Errors
///
/// Returns an error if the frame is too short, the header bytes do not match
/// the observed Auto Start response format, or the flag byte is unknown.
pub fn autostart_from_response(bytes: &[u8]) -> Result<bool> {
    if bytes.len() < 9 {
        return Err(Error::ProtocolDecode(
            "invalid Auto Start response: frame too short".to_string(),
        ));
    }

    if bytes[0] != 0x00 || bytes[1] != 0x08 || bytes[2] != 0x87 || bytes[3] != 0x24 {
        return Err(Error::ProtocolDecode(
            "invalid Auto Start response: header mismatch".to_string(),
        ));
    }

    if bytes[4] != 0x01 {
        return Err(Error::ProtocolDecode(
            "invalid Auto Start response: setting ID mismatch".to_string(),
        ));
    }

    match bytes[5] {
        0x00 => Ok(false),
        0x01 => Ok(true),
        _ => {
            Err(Error::ProtocolDecode("invalid Auto Start response: unknown flag byte".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AUTOSTART_DISABLE_COMMAND, AUTOSTART_ENABLE_COMMAND, LOAD_AUTOSTART_COMMAND,
        SMARTGESTURE_DISABLE_COMMAND, SMARTGESTURE_ENABLE_COMMAND, autostart_command,
        autostart_from_response, smartgesture_command,
    };

    #[test]
    fn smartgesture_command_selects_correct_constant() {
        assert_eq!(smartgesture_command(true), &SMARTGESTURE_ENABLE_COMMAND);
        assert_eq!(smartgesture_command(false), &SMARTGESTURE_DISABLE_COMMAND);
    }

    #[test]
    fn autostart_command_selects_correct_constant() {
        assert_eq!(autostart_command(true), &AUTOSTART_ENABLE_COMMAND);
        assert_eq!(autostart_command(false), &AUTOSTART_DISABLE_COMMAND);
    }

    #[test]
    fn keeps_smartgesture_commands_stable() {
        assert_eq!(
            SMARTGESTURE_ENABLE_COMMAND,
            [0x00, 0xC9, 0x47, 0x24, 0x04, 0x01, 0x00, 0x00, 0x3C]
        );
        assert_eq!(
            SMARTGESTURE_DISABLE_COMMAND,
            [0x00, 0xC9, 0x47, 0x24, 0x04, 0x00, 0x00, 0x00, 0x57]
        );
    }

    #[test]
    fn keeps_autostart_commands_stable() {
        assert_eq!(
            AUTOSTART_ENABLE_COMMAND,
            [0x00, 0xC9, 0x47, 0x24, 0x01, 0x01, 0x00, 0x00, 0x3F]
        );
        assert_eq!(
            AUTOSTART_DISABLE_COMMAND,
            [0x00, 0xC9, 0x47, 0x24, 0x01, 0x00, 0x00, 0x00, 0x54]
        );
    }

    #[test]
    fn keeps_load_autostart_command_stable() {
        assert_eq!(LOAD_AUTOSTART_COMMAND, [0x00, 0xC9, 0x07, 0x24, 0x01, 0x00, 0x00, 0x00, 0x22]);
    }

    #[test]
    fn parses_autostart_enabled_response() {
        let enabled =
            autostart_from_response(&[0x00, 0x08, 0x87, 0x24, 0x01, 0x01, 0x00, 0x00, 0xA5]);
        assert!(enabled.unwrap());
    }

    #[test]
    fn parses_autostart_disabled_response() {
        let disabled =
            autostart_from_response(&[0x00, 0x08, 0x87, 0x24, 0x01, 0x00, 0x00, 0x00, 0x00]);
        assert!(!disabled.unwrap());
    }

    #[test]
    fn rejects_short_autostart_response() {
        assert!(autostart_from_response(&[0x00, 0x08, 0x87]).is_err());
    }

    #[test]
    fn rejects_invalid_autostart_header() {
        assert!(
            autostart_from_response(&[0x00, 0x00, 0x87, 0x24, 0x01, 0x01, 0x00, 0x00, 0xA5])
                .is_err()
        );
    }

    #[test]
    fn rejects_mismatched_autostart_setting_id() {
        assert!(
            autostart_from_response(&[0x00, 0x08, 0x87, 0x24, 0x02, 0x01, 0x00, 0x00, 0xA5])
                .is_err()
        );
    }

    #[test]
    fn rejects_unknown_autostart_flag() {
        assert!(
            autostart_from_response(&[0x00, 0x08, 0x87, 0x24, 0x01, 0xFF, 0x00, 0x00, 0xA5])
                .is_err()
        );
    }
}
