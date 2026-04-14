use async_trait::async_trait;

use crate::Result;

/// Transport backend kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    /// Bluetooth Low Energy transport.
    Ble,
    /// USB transport.
    Usb,
}

/// Minimal transport contract shared by IQOS backends.
///
/// This trait is intentionally protocol-oriented rather than deeply BLE-shaped.
/// Backends may internally use BLE characteristics, USB endpoints, or other
/// framing details, but the library core should interact in terms of command and
/// response frames.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Return the transport kind.
    fn kind(&self) -> TransportKind;

    /// Send a command that expects a single response frame.
    async fn request(&self, command: &[u8]) -> Result<Vec<u8>>;

    /// Send a command that does not require an immediate response frame.
    async fn send(&self, command: &[u8]) -> Result<()>;
}
