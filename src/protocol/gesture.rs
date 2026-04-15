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

#[cfg(test)]
mod tests {
    use super::{
        AUTOSTART_DISABLE_COMMAND, AUTOSTART_ENABLE_COMMAND, SMARTGESTURE_DISABLE_COMMAND,
        SMARTGESTURE_ENABLE_COMMAND, autostart_command, smartgesture_command,
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
}
