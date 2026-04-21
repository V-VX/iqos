use crate::{Error, Result};

/// Product number request for the stick/vape unit.
pub const PRODUCT_NUMBER_COMMAND: [u8; 5] = [0x00, 0xC0, 0x00, 0x03, 0x09];

/// Product number request for the holder unit.
pub const HOLDER_PRODUCT_NUMBER_COMMAND: [u8; 5] = [0x00, 0xC9, 0x00, 0x03, 0x09];

/// IQOS product-number target kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductNumberKind {
    /// Product number reported by the stick/vape unit.
    Stick,
    /// Product number reported by the holder/charger unit.
    Holder,
}

impl ProductNumberKind {
    /// Return the request command for this product-number target.
    #[must_use]
    pub const fn command(self) -> &'static [u8] {
        match self {
            Self::Stick => &PRODUCT_NUMBER_COMMAND,
            Self::Holder => &HOLDER_PRODUCT_NUMBER_COMMAND,
        }
    }

    const fn response_prefix(self) -> [u8; 4] {
        match self {
            Self::Stick => [0x00, 0xC0, 0x88, 0x03],
            Self::Holder => [0x00, 0x08, 0x88, 0x03],
        }
    }
}

/// Parse a product-number response into a printable ASCII string.
///
/// The legacy CLI slices stick responses as bytes `4..len - 1`, treating the
/// final byte as a trailing checksum/status byte. Holder responses are sliced
/// from byte 4 to the end.
///
/// # Errors
///
/// Returns an error if the frame is too short, has an unexpected header, or
/// contains no product-number payload.
pub fn product_number_from_response(bytes: &[u8], kind: ProductNumberKind) -> Result<String> {
    let prefix = kind.response_prefix();
    if bytes.len() < prefix.len() {
        return Err(Error::ProtocolDecode(
            "invalid product number response: frame too short".to_string(),
        ));
    }

    if bytes[..prefix.len()] != prefix {
        return Err(Error::ProtocolDecode(
            "invalid product number response: header mismatch".to_string(),
        ));
    }

    let payload = match kind {
        ProductNumberKind::Stick => {
            if bytes.len() <= prefix.len() + 1 {
                return Err(Error::ProtocolDecode(
                    "invalid product number response: missing stick payload".to_string(),
                ));
            }
            &bytes[prefix.len()..bytes.len() - 1]
        }
        ProductNumberKind::Holder => {
            if bytes.len() <= prefix.len() {
                return Err(Error::ProtocolDecode(
                    "invalid product number response: missing holder payload".to_string(),
                ));
            }
            &bytes[prefix.len()..]
        }
    };

    Ok(payload
        .iter()
        .map(|&byte| if byte.is_ascii() && !byte.is_ascii_control() { byte as char } else { '.' })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{
        HOLDER_PRODUCT_NUMBER_COMMAND, PRODUCT_NUMBER_COMMAND, ProductNumberKind,
        product_number_from_response,
    };

    #[test]
    fn parses_stick_product_number_without_trailing_byte() {
        let bytes = [&[0x00, 0xC0, 0x88, 0x03][..], b"ABCD123456", &[0xAA]].concat();

        let product_number = product_number_from_response(&bytes, ProductNumberKind::Stick)
            .expect("stick product number should parse");

        assert_eq!(product_number, "ABCD123456");
    }

    #[test]
    fn parses_holder_product_number_to_end_of_frame() {
        let bytes = [&[0x00, 0x08, 0x88, 0x03][..], b"HOLDER123456"].concat();

        let product_number = product_number_from_response(&bytes, ProductNumberKind::Holder)
            .expect("holder product number should parse");

        assert_eq!(product_number, "HOLDER123456");
    }

    #[test]
    fn replaces_non_printable_payload_bytes_with_dots() {
        let product_number = product_number_from_response(
            &[0x00, 0x08, 0x88, 0x03, b'A', 0x00, 0xFF],
            ProductNumberKind::Holder,
        )
        .expect("holder product number should parse");

        assert_eq!(product_number, "A..");
    }

    #[test]
    fn rejects_invalid_product_number_header() {
        let result = product_number_from_response(
            &[0x00, 0xC9, 0x88, 0x03, b'A', 0xAA],
            ProductNumberKind::Stick,
        );

        assert!(result.is_err());
    }

    #[test]
    fn keeps_product_number_commands_stable() {
        assert_eq!(PRODUCT_NUMBER_COMMAND, [0x00, 0xC0, 0x00, 0x03, 0x09]);
        assert_eq!(HOLDER_PRODUCT_NUMBER_COMMAND, [0x00, 0xC9, 0x00, 0x03, 0x09]);
    }
}
