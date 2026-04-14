use thiserror::Error;

/// Crate-wide result type.
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for the IQOS library scaffold.
#[derive(Debug, Error)]
pub enum Error {
    /// Returned when a transport backend reports an error.
    #[error("transport error: {0}")]
    Transport(String),

    /// Returned when protocol encoding fails.
    #[error("protocol encode error: {0}")]
    ProtocolEncode(String),

    /// Returned when protocol decoding fails.
    #[error("protocol decode error: {0}")]
    ProtocolDecode(String),

    /// Returned when a requested operation is not supported by the current model or backend.
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}
