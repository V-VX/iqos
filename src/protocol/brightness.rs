use core::{fmt, str::FromStr};

use crate::{Error, Result};

/// Command used to request the current brightness configuration.
pub const LOAD_BRIGHTNESS_COMMAND: [u8; 5] = [0x00, 0xC0, 0x02, 0x23, 0xC3];

/// Command sequence used to switch brightness to `High`.
pub const SET_BRIGHTNESS_HIGH_COMMANDS: [&[u8]; 3] = [
    &[0x00, 0xC0, 0x46, 0x23, 0x64, 0x00, 0x00, 0x00, 0x4F],
    &[0x00, 0xC0, 0x02, 0x23, 0xC3],
    &[0x00, 0xC9, 0x44, 0x24, 0x64, 0x00, 0x00, 0x00, 0x34],
];

/// Command sequence used to switch brightness to `Low`.
pub const SET_BRIGHTNESS_LOW_COMMANDS: [&[u8]; 3] = [
    &[0x00, 0xC0, 0x46, 0x23, 0x1E, 0x00, 0x00, 0x00, 0xE1],
    &[0x00, 0xC0, 0x02, 0x23, 0xC3],
    &[0x00, 0xC9, 0x44, 0x24, 0x1E, 0x00, 0x00, 0x00, 0x9A],
];

/// Brightness setting supported by IQOS devices that expose LED brightness control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrightnessLevel {
    /// High brightness.
    High,
    /// Low brightness.
    Low,
}

impl BrightnessLevel {
    /// Parse a brightness level from a protocol response frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the frame is too short, has an unexpected header, or
    /// contains an unknown payload flag.
    pub fn from_response(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 9 {
            return Err(Error::ProtocolDecode(
                "invalid brightness response: frame too short".to_string(),
            ));
        }

        if bytes[0] != 0x00 || bytes[1] != 0xC0 || bytes[2] != 0x86 || bytes[3] != 0x23 {
            return Err(Error::ProtocolDecode(
                "invalid brightness response: header mismatch".to_string(),
            ));
        }

        match bytes[4] {
            0x64 => Ok(Self::High),
            0x1E => Ok(Self::Low),
            _ => Err(Error::ProtocolDecode(
                "invalid brightness response: unknown level flag".to_string(),
            )),
        }
    }

    /// Return the write sequence required to set this brightness level.
    #[must_use]
    pub const fn write_commands(self) -> &'static [&'static [u8]; 3] {
        match self {
            Self::High => &SET_BRIGHTNESS_HIGH_COMMANDS,
            Self::Low => &SET_BRIGHTNESS_LOW_COMMANDS,
        }
    }

    /// Return the user-facing lowercase label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Low => "low",
        }
    }
}

impl FromStr for BrightnessLevel {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "high" => Ok(Self::High),
            "low" => Ok(Self::Low),
            _ => Err(Error::ProtocolDecode("invalid brightness level string".to_string())),
        }
    }
}

impl fmt::Display for BrightnessLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BrightnessLevel, LOAD_BRIGHTNESS_COMMAND, SET_BRIGHTNESS_HIGH_COMMANDS,
        SET_BRIGHTNESS_LOW_COMMANDS,
    };

    #[test]
    fn parses_high_brightness_response() {
        let value = BrightnessLevel::from_response(&[0x00, 0xC0, 0x86, 0x23, 0x64, 0, 0, 0, 0]);
        assert_eq!(value.unwrap(), BrightnessLevel::High);
    }

    #[test]
    fn parses_low_brightness_response() {
        let value = BrightnessLevel::from_response(&[0x00, 0xC0, 0x86, 0x23, 0x1E, 0, 0, 0, 0]);
        assert_eq!(value.unwrap(), BrightnessLevel::Low);
    }

    #[test]
    fn rejects_invalid_brightness_header() {
        let error = BrightnessLevel::from_response(&[0x00, 0x00, 0x86, 0x23, 0x64, 0, 0, 0, 0]);
        assert!(error.is_err());
    }

    #[test]
    fn returns_expected_write_sequences() {
        assert_eq!(BrightnessLevel::High.write_commands(), &SET_BRIGHTNESS_HIGH_COMMANDS);
        assert_eq!(BrightnessLevel::Low.write_commands(), &SET_BRIGHTNESS_LOW_COMMANDS);
    }

    #[test]
    fn keeps_load_command_stable() {
        assert_eq!(LOAD_BRIGHTNESS_COMMAND, [0x00, 0xC0, 0x02, 0x23, 0xC3]);
    }
}
