//! btleplug backend scaffold.
//!
//! The concrete BLE transport implementation will be extracted incrementally from
//! `iqos_cli` after the transport contract and protocol/domain types are settled.

use async_trait::async_trait;

use crate::{
    Result,
    error::Error,
    transport::{Transport, TransportKind},
};

/// Placeholder btleplug-backed BLE transport.
#[derive(Debug, Default)]
pub struct BtleplugTransport;

#[async_trait]
impl Transport for BtleplugTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Ble
    }

    async fn request(&self, _command: &[u8]) -> Result<Vec<u8>> {
        Err(Error::Unsupported("btleplug transport scaffold is not implemented yet".to_string()))
    }

    async fn send(&self, _command: &[u8]) -> Result<()> {
        Err(Error::Unsupported("btleplug transport scaffold is not implemented yet".to_string()))
    }
}
