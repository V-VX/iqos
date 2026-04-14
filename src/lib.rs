//! # iqos
//!
//! Rust library scaffold for controlling IQOS devices.
//!
//! This crate is being prepared as the reusable library layer extracted from the
//! existing `iqos_cli` implementation. The intended architecture keeps protocol
//! and device logic in the library core, while interactive CLI concerns stay out
//! of the public API.
//!
//! ## Planned layers
//!
//! 1. [`protocol`] for command builders, response parsers, and typed domain values
//! 2. [`transport`] for the transport contract shared by backends
//! 3. [`transports`] for backend implementations such as BLE and future USB
//!
//! ## Features
//!
//! - `btleplug-support`: enables the BLE transport backend integration scaffold
//! - `usb-support`: reserved for future USB transport support

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Error types and the crate-wide `Result` alias.
pub mod error;
/// Protocol-layer command, response, and domain types.
pub mod protocol;
/// Transport abstraction shared by backend implementations.
pub mod transport;
/// Backend transport implementations and scaffolds.
pub mod transports;

pub use error::{Error, Result};
pub use protocol::{
    BrightnessLevel, CommandFrame, DeviceCapability, DeviceModel, FirmwareKind, FirmwareVersion,
    ResponseFrame,
};
pub use transport::{Transport, TransportKind};

/// Library facade placeholder for future extracted IQOS session/device API.
///
/// The concrete device/session shape will be introduced incrementally as logic
/// is extracted from `iqos_cli` into transport-agnostic and backend-specific
/// layers.
#[derive(Debug)]
pub struct Iqos<T: Transport> {
    transport: T,
}

impl<T: Transport> Iqos<T> {
    /// Create a new IQOS library facade around a transport implementation.
    #[must_use]
    pub const fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Borrow the underlying transport.
    #[must_use]
    pub const fn transport(&self) -> &T {
        &self.transport
    }

    /// Mutably borrow the underlying transport.
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }
}
