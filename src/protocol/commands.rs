/// Opaque command frame passed to a transport backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFrame {
    bytes: Vec<u8>,
}

impl CommandFrame {
    /// Create a command frame from raw bytes.
    #[must_use]
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self { bytes: bytes.into() }
    }

    /// Borrow the raw command bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::CommandFrame;

    #[test]
    fn command_frame_preserves_bytes() {
        let frame = CommandFrame::new([0x01, 0x02, 0x03]);
        assert_eq!(frame.as_bytes(), &[0x01, 0x02, 0x03]);
    }
}
