use core::fmt;

use crate::{Error, Result};

/// Command used to request the firmware version for the stick/vape unit.
pub const LOAD_STICK_FIRMWARE_VERSION_COMMAND: [u8; 7] = [0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00];

/// Command used to request the firmware version for the holder unit.
pub const LOAD_HOLDER_FIRMWARE_VERSION_COMMAND: [u8; 7] =
    [0x00, 0xC9, 0x00, 0x00, 0x00, 0x00, 0x00];

/// IQOS firmware target kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareKind {
    /// Firmware running on the stick/vape device.
    Stick = 0xC0,
    /// Firmware running on the holder/charger device.
    Holder = 0x08,
}

/// Parsed firmware version.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirmwareVersion {
    /// Major version component.
    pub major: u8,
    /// Minor version component.
    pub minor: u8,
    /// Patch version component.
    pub patch: u8,
    /// Year/build component.
    pub year: u8,
}

impl FirmwareVersion {
    /// Parse a firmware version frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the frame is too short or does not match the
    /// expected protocol header for the requested firmware kind.
    pub fn from_response(bytes: &[u8], kind: FirmwareKind) -> Result<Self> {
        if bytes.len() < 10 {
            return Err(Error::ProtocolDecode(
                "invalid firmware response: frame too short".to_string(),
            ));
        }

        if bytes[0] != 0x00 || bytes[1] != kind as u8 || bytes[2] != 0x88 || bytes[3] != 0x00 {
            return Err(Error::ProtocolDecode(
                "invalid firmware response: header mismatch".to_string(),
            ));
        }

        Ok(Self { major: bytes[6], minor: bytes[7], patch: bytes[8], year: bytes[9] })
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "v{}.{}.{}.{}", self.major, self.minor, self.patch, self.year)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FirmwareKind, FirmwareVersion, LOAD_HOLDER_FIRMWARE_VERSION_COMMAND,
        LOAD_STICK_FIRMWARE_VERSION_COMMAND,
    };

    #[test]
    fn parses_stick_firmware_response() {
        let version = FirmwareVersion::from_response(
            &[0x00, 0xC0, 0x88, 0x00, 0x00, 0x00, 0x02, 0x05, 0x07, 0x18],
            FirmwareKind::Stick,
        )
        .unwrap();

        assert_eq!(version, FirmwareVersion { major: 2, minor: 5, patch: 7, year: 24 });
    }

    #[test]
    fn parses_holder_firmware_response() {
        let version = FirmwareVersion::from_response(
            &[0x00, 0x08, 0x88, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x19],
            FirmwareKind::Holder,
        )
        .unwrap();

        assert_eq!(version.to_string(), "v1.2.3.25");
    }

    #[test]
    fn rejects_invalid_firmware_header() {
        let result = FirmwareVersion::from_response(
            &[0x00, 0xC9, 0x88, 0x00, 0x00, 0x00, 0x02, 0x05, 0x07, 0x18],
            FirmwareKind::Stick,
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_short_firmware_response() {
        let result = FirmwareVersion::from_response(&[0x00, 0xC0, 0x88, 0x00], FirmwareKind::Stick);

        assert!(result.is_err());
    }

    #[test]
    fn keeps_firmware_commands_stable() {
        assert_eq!(LOAD_STICK_FIRMWARE_VERSION_COMMAND, [0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(
            LOAD_HOLDER_FIRMWARE_VERSION_COMMAND,
            [0x00, 0xC9, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }
}
