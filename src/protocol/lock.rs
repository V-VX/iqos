/// First command in the lock sequence.
pub(crate) const LOCK_COMMAND_1: [u8; 9] = [0x00, 0xC9, 0x44, 0x04, 0x02, 0xFF, 0x00, 0x00, 0x5A];

/// Second command in the lock sequence (shared with unlock).
pub(crate) const LOCK_COMMAND_2: [u8; 5] = [0x00, 0xC9, 0x00, 0x04, 0x1C];

/// First command in the unlock sequence.
pub(crate) const UNLOCK_COMMAND_1: [u8; 9] = [0x00, 0xC9, 0x44, 0x04, 0x00, 0x00, 0x00, 0x00, 0x5D];

/// Second command in the unlock sequence (shared with lock).
pub(crate) const UNLOCK_COMMAND_2: [u8; 5] = [0x00, 0xC9, 0x00, 0x04, 0x1C];

/// Confirmation command sent after both lock and unlock sequences.
pub(crate) const CONFIRMATION_COMMAND: [u8; 5] = [0x00, 0xC0, 0x01, 0x00, 0xF6];

/// Return the three-command lock sequence.
#[must_use]
pub fn lock_commands() -> [&'static [u8]; 3] {
    [&LOCK_COMMAND_1, &LOCK_COMMAND_2, &CONFIRMATION_COMMAND]
}

/// Return the three-command unlock sequence.
#[must_use]
pub fn unlock_commands() -> [&'static [u8]; 3] {
    [&UNLOCK_COMMAND_1, &UNLOCK_COMMAND_2, &CONFIRMATION_COMMAND]
}

#[cfg(test)]
mod tests {
    use super::{
        CONFIRMATION_COMMAND, LOCK_COMMAND_1, LOCK_COMMAND_2, UNLOCK_COMMAND_1, UNLOCK_COMMAND_2,
        lock_commands, unlock_commands,
    };

    #[test]
    fn keeps_lock_commands_stable() {
        assert_eq!(LOCK_COMMAND_1, [0x00, 0xC9, 0x44, 0x04, 0x02, 0xFF, 0x00, 0x00, 0x5A]);
        assert_eq!(LOCK_COMMAND_2, [0x00, 0xC9, 0x00, 0x04, 0x1C]);
    }

    #[test]
    fn keeps_unlock_commands_stable() {
        assert_eq!(UNLOCK_COMMAND_1, [0x00, 0xC9, 0x44, 0x04, 0x00, 0x00, 0x00, 0x00, 0x5D]);
        assert_eq!(UNLOCK_COMMAND_2, [0x00, 0xC9, 0x00, 0x04, 0x1C]);
    }

    #[test]
    fn keeps_confirmation_command_stable() {
        assert_eq!(CONFIRMATION_COMMAND, [0x00, 0xC0, 0x01, 0x00, 0xF6]);
    }

    #[test]
    fn lock_and_unlock_share_secondary_command() {
        assert_eq!(LOCK_COMMAND_2, UNLOCK_COMMAND_2);
    }

    #[test]
    fn lock_commands_returns_correct_sequence() {
        let cmds = lock_commands();
        assert_eq!(cmds[0], &LOCK_COMMAND_1);
        assert_eq!(cmds[1], &LOCK_COMMAND_2);
        assert_eq!(cmds[2], &CONFIRMATION_COMMAND);
    }

    #[test]
    fn unlock_commands_returns_correct_sequence() {
        let cmds = unlock_commands();
        assert_eq!(cmds[0], &UNLOCK_COMMAND_1);
        assert_eq!(cmds[1], &UNLOCK_COMMAND_2);
        assert_eq!(cmds[2], &CONFIRMATION_COMMAND);
    }
}
