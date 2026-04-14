/// Opaque response frame returned by a transport backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseFrame {
    bytes: Vec<u8>,
}

impl ResponseFrame {
    /// Create a response frame from raw bytes.
    #[must_use]
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self { bytes: bytes.into() }
    }

    /// Borrow the raw response bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::ResponseFrame;

    #[test]
    fn response_frame_preserves_bytes() {
        let frame = ResponseFrame::new([0xAA, 0xBB]);
        assert_eq!(frame.as_bytes(), &[0xAA, 0xBB]);
    }
}
