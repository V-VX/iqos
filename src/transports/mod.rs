//! Backend transport implementations.
//!
//! This module contains transport-specific adapters. The initial extraction work
//! will focus on a BLE backend built around `btleplug`, while reserving a clean
//! path for future USB support.

#[cfg(feature = "btleplug-support")]
pub mod ble_btleplug;

#[cfg(feature = "usb-support")]
pub mod usb;
