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

/// BLE-specific IQOS session and metadata logic.
#[cfg(feature = "btleplug-support")]
pub mod ble;
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
    BrightnessLevel, DeviceCapability, DeviceInfo, DeviceModel, DiagnosticData, FirmwareKind,
    FirmwareVersion, FlexBatteryMode, FlexBatterySettings, FlexPuffSetting, VibrationSettings,
};
pub use transport::{Transport, TransportKind};

#[cfg(feature = "btleplug-support")]
pub use ble::IqosBle;

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

    /// Read the current device brightness setting.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport request fails or the response frame
    /// cannot be decoded as a brightness response.
    pub async fn read_brightness(&self) -> Result<BrightnessLevel> {
        let response = self.transport.request(&protocol::LOAD_BRIGHTNESS_COMMAND).await?;
        BrightnessLevel::from_response(&response)
    }

    /// Update the device brightness setting.
    ///
    /// Sends the three-command write sequence for the requested level.
    ///
    /// # Errors
    ///
    /// Returns an error if any transport send fails.
    pub async fn set_brightness(&self, level: BrightnessLevel) -> Result<()> {
        for command in level.write_commands() {
            self.transport.send(command).await?;
        }
        Ok(())
    }

    /// Read the current `FlexPuff` setting.
    ///
    /// `FlexPuff` is supported on ILUMA and ILUMA i holder-based models.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport request fails or the response frame
    /// cannot be decoded as a `FlexPuff` response.
    pub async fn read_flexpuff(&self) -> Result<FlexPuffSetting> {
        let response = self.transport.request(&protocol::LOAD_FLEXPUFF_COMMAND).await?;
        FlexPuffSetting::from_response(&response)
    }

    /// Update the `FlexPuff` setting.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport send fails.
    pub async fn set_flexpuff(&self, setting: FlexPuffSetting) -> Result<()> {
        self.transport.send(setting.write_command()).await
    }

    /// Enable or disable Smart Gesture for the provided model.
    ///
    /// Smart Gesture is only supported on holder-based ILUMA and ILUMA i models.
    ///
    /// # Errors
    ///
    /// Returns an error if the model does not support Smart Gesture, or if the
    /// transport send fails.
    pub async fn set_smartgesture(&self, model: DeviceModel, enabled: bool) -> Result<()> {
        if !model.supports(DeviceCapability::SmartGesture) {
            return Err(Error::Unsupported(format!(
                "Smart Gesture is not supported for model {model:?}"
            )));
        }
        self.transport.send(protocol::smartgesture_command(enabled)).await
    }

    /// Lock the device.
    ///
    /// Sends the two-command lock sequence followed by a confirmation.
    ///
    /// # Errors
    ///
    /// Returns an error if the model does not support device lock, or if any
    /// transport send fails.
    pub async fn lock(&self, model: DeviceModel) -> Result<()> {
        if !model.supports(DeviceCapability::DeviceLock) {
            return Err(Error::Unsupported(format!(
                "device lock is not supported for model {model:?}"
            )));
        }
        for command in protocol::lock_commands() {
            self.transport.send(command).await?;
        }
        Ok(())
    }

    /// Unlock the device.
    ///
    /// Sends the two-command unlock sequence followed by a confirmation.
    ///
    /// # Errors
    ///
    /// Returns an error if the model does not support device lock, or if any
    /// transport send fails.
    pub async fn unlock(&self, model: DeviceModel) -> Result<()> {
        if !model.supports(DeviceCapability::DeviceLock) {
            return Err(Error::Unsupported(format!(
                "device lock is not supported for model {model:?}"
            )));
        }
        for command in protocol::unlock_commands() {
            self.transport.send(command).await?;
        }
        Ok(())
    }

    /// Enable or disable Auto Start for the provided model.
    ///
    /// Auto Start is only supported on holder-based ILUMA and ILUMA i models.
    ///
    /// # Errors
    ///
    /// Returns an error if the model does not support Auto Start, or if the
    /// transport send fails.
    pub async fn set_autostart(&self, model: DeviceModel, enabled: bool) -> Result<()> {
        if !model.supports(DeviceCapability::AutoStart) {
            return Err(Error::Unsupported(format!(
                "Auto Start is not supported for model {model:?}"
            )));
        }
        self.transport.send(protocol::autostart_command(enabled)).await
    }

    /// Read the current `FlexBattery` settings including mode and Pause Mode state.
    ///
    /// `FlexBattery` is supported on IQOS ILUMA i holder-based models.
    ///
    /// # Errors
    ///
    /// Returns an error if the model does not support `FlexBattery`, a transport
    /// request fails, or a response frame cannot be decoded.
    pub async fn read_flexbattery(&self, model: DeviceModel) -> Result<FlexBatterySettings> {
        if !model.supports(DeviceCapability::FlexBattery) {
            return Err(Error::Unsupported(format!(
                "FlexBattery is not supported for model {model:?}"
            )));
        }
        let mode_response = self.transport.request(&protocol::LOAD_FLEXBATTERY_COMMAND).await?;
        let pause_response = self.transport.request(&protocol::LOAD_PAUSEMODE_COMMAND).await?;
        FlexBatterySettings::from_responses(&mode_response, &pause_response)
    }

    /// Update the `FlexBattery` settings.
    ///
    /// Always sends the mode write sequence. When `settings.pause_mode()` is
    /// `Some`, also sends the Pause Mode write sequence.
    ///
    /// # Errors
    ///
    /// Returns an error if the model does not support `FlexBattery`, or if any
    /// transport send fails.
    pub async fn set_flexbattery(
        &self,
        model: DeviceModel,
        settings: FlexBatterySettings,
    ) -> Result<()> {
        if !model.supports(DeviceCapability::FlexBattery) {
            return Err(Error::Unsupported(format!(
                "FlexBattery is not supported for model {model:?}"
            )));
        }
        self.transport.send(settings.mode().write_command()).await?;
        self.transport.send(&protocol::LOAD_FLEXBATTERY_COMMAND).await?;
        if let Some(pause_mode) = settings.pause_mode() {
            self.transport.send(protocol::pausemode_command(pause_mode)).await?;
            self.transport.send(&protocol::LOAD_PAUSEMODE_COMMAND).await?;
        }
        Ok(())
    }

    /// Read diagnostic telemetry from the device.
    ///
    /// Sends all diagnosis commands in sequence and accumulates the parsed
    /// responses into a [`DiagnosticData`] value.
    ///
    /// # Errors
    ///
    /// Returns an error if any transport request fails, or if a response frame
    /// that matches a known header cannot be decoded.
    pub async fn read_diagnosis(&self) -> Result<DiagnosticData> {
        let mut builder = protocol::DiagnosticDataBuilder::default();
        for command in protocol::ALL_DIAGNOSIS_COMMANDS {
            let response = self.transport.request(command).await?;
            builder = builder.parse(&response)?;
        }
        Ok(builder.build())
    }

    /// Read the firmware version for the selected IQOS component.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport request fails or the response frame
    /// cannot be decoded as a firmware response for the requested kind.
    pub async fn read_firmware_version(&self, kind: FirmwareKind) -> Result<FirmwareVersion> {
        let command = match kind {
            FirmwareKind::Stick => &protocol::LOAD_STICK_FIRMWARE_VERSION_COMMAND,
            FirmwareKind::Holder => &protocol::LOAD_HOLDER_FIRMWARE_VERSION_COMMAND,
        };
        let response = self.transport.request(command).await?;
        FirmwareVersion::from_response(&response, kind)
    }

    /// Read the current vibration settings for the provided model.
    ///
    /// Holder-based models require an additional request to retrieve the
    /// charge-start vibration flag. One-piece models only use the main
    /// vibration settings frame.
    ///
    /// # Errors
    ///
    /// Returns an error if vibration is unsupported for the model, a transport
    /// operation fails, or a response frame cannot be decoded.
    pub async fn read_vibration_settings(&self, model: DeviceModel) -> Result<VibrationSettings> {
        if !model.supports(DeviceCapability::Vibration) {
            return Err(Error::Unsupported(format!(
                "vibration is not supported for model {model:?}"
            )));
        }

        let charge_start = if model.supports_charge_start_vibration() {
            let response =
                self.transport.request(&protocol::LOAD_VIBRATE_CHARGE_START_COMMAND).await?;
            Some(VibrationSettings::charge_start_from_response(&response)?)
        } else {
            None
        };

        let response = self.transport.request(&protocol::LOAD_VIBRATION_SETTINGS_COMMAND).await?;
        let settings = VibrationSettings::from_response(&response, model)?;

        Ok(charge_start.map_or(settings, |value| settings.with_observed_charge_start(value)))
    }

    /// Update vibration settings for the provided model.
    ///
    /// # Errors
    ///
    /// Returns an error if the model does not support the requested vibration
    /// fields or a transport write fails.
    pub async fn update_vibration_settings(
        &self,
        model: DeviceModel,
        settings: VibrationSettings,
    ) -> Result<()> {
        let settings = if model.supports_charge_start_vibration()
            && settings.when_charging_start().is_none()
        {
            let response =
                self.transport.request(&protocol::LOAD_VIBRATE_CHARGE_START_COMMAND).await?;
            settings.with_observed_charge_start(VibrationSettings::charge_start_from_response(
                &response,
            )?)
        } else {
            settings
        };

        for command in settings.build_update_commands(model)? {
            self.transport.send(&command).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Mutex, MutexGuard},
    };

    use async_trait::async_trait;
    use futures::executor::block_on;

    use super::*;

    #[derive(Debug)]
    struct MockTransport {
        requests: Mutex<Vec<Vec<u8>>>,
        sends: Mutex<Vec<Vec<u8>>>,
        responses: Mutex<VecDeque<Result<Vec<u8>>>>,
        send_results: Mutex<VecDeque<Result<()>>>,
    }

    impl MockTransport {
        fn with_responses(responses: impl IntoIterator<Item = Result<Vec<u8>>>) -> Self {
            Self {
                requests: Mutex::new(Vec::new()),
                sends: Mutex::new(Vec::new()),
                responses: Mutex::new(responses.into_iter().collect()),
                send_results: Mutex::new(VecDeque::new()),
            }
        }

        fn recorded_requests(&self) -> MutexGuard<'_, Vec<Vec<u8>>> {
            self.requests.lock().expect("request log mutex poisoned")
        }

        fn recorded_sends(&self) -> MutexGuard<'_, Vec<Vec<u8>>> {
            self.sends.lock().expect("send log mutex poisoned")
        }

        fn with_send_results(mut self, results: impl IntoIterator<Item = Result<()>>) -> Self {
            self.send_results = Mutex::new(results.into_iter().collect());
            self
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        fn kind(&self) -> TransportKind {
            TransportKind::Ble
        }

        async fn request(&self, command: &[u8]) -> Result<Vec<u8>> {
            self.requests.lock().expect("request log mutex poisoned").push(command.to_vec());

            self.responses
                .lock()
                .expect("response queue mutex poisoned")
                .pop_front()
                .unwrap_or_else(|| Err(Error::Transport("missing queued response".to_string())))
        }

        async fn send(&self, command: &[u8]) -> Result<()> {
            self.sends.lock().expect("send log mutex poisoned").push(command.to_vec());

            self.send_results
                .lock()
                .expect("send result queue mutex poisoned")
                .pop_front()
                .unwrap_or(Ok(()))
        }
    }

    #[test]
    fn read_brightness_uses_expected_request_and_parses_response() {
        let transport = MockTransport::with_responses([Ok(vec![
            0x00, 0xC0, 0x86, 0x23, 0x64, 0x00, 0x00, 0x00, 0x00,
        ])]);
        let iqos = Iqos::new(transport);

        let brightness = block_on(iqos.read_brightness()).expect("brightness should parse");

        assert_eq!(brightness, BrightnessLevel::High);
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[protocol::LOAD_BRIGHTNESS_COMMAND.to_vec()],
        );
    }

    #[test]
    fn read_firmware_version_uses_expected_request_and_parses_response() {
        let transport = MockTransport::with_responses([Ok(vec![
            0x00, 0x08, 0x88, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x19,
        ])]);
        let iqos = Iqos::new(transport);

        let firmware = block_on(iqos.read_firmware_version(FirmwareKind::Holder))
            .expect("firmware should parse");

        assert_eq!(firmware, FirmwareVersion { major: 1, minor: 2, patch: 3, year: 25 });
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[protocol::LOAD_HOLDER_FIRMWARE_VERSION_COMMAND.to_vec()],
        );
    }

    #[test]
    fn read_brightness_propagates_transport_errors() {
        let iqos = Iqos::new(MockTransport::with_responses([Err(Error::Transport(
            "transport down".to_string(),
        ))]));

        let error = block_on(iqos.read_brightness()).expect_err("transport error should surface");

        assert!(matches!(error, Error::Transport(message) if message == "transport down"));
    }

    #[test]
    fn read_firmware_version_propagates_decode_errors() {
        let iqos = Iqos::new(MockTransport::with_responses([Ok(vec![
            0x00, 0xC9, 0x88, 0x00, 0x00, 0x00, 0x02, 0x05, 0x07, 0x18,
        ])]));

        let error = block_on(iqos.read_firmware_version(FirmwareKind::Stick))
            .expect_err("invalid firmware frame should fail");

        assert!(
            matches!(error, Error::ProtocolDecode(message) if message.contains("header mismatch"))
        );
    }

    #[test]
    fn read_vibration_settings_for_one_piece_model_uses_single_request() {
        let transport = MockTransport::with_responses([Ok(vec![
            0x00, 0x08, 0x84, 0x23, 0x10, 0x00, 0x01, 0x10, 0x77,
        ])]);
        let iqos = Iqos::new(transport);

        let settings = block_on(iqos.read_vibration_settings(DeviceModel::IlumaOne))
            .expect("one-piece vibration should parse");

        assert_eq!(settings, VibrationSettings::new(true, false, false, true));
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[protocol::LOAD_VIBRATION_SETTINGS_COMMAND.to_vec()],
        );
    }

    #[test]
    fn read_vibration_settings_for_holder_model_reads_charge_start_first() {
        let transport = MockTransport::with_responses([
            Ok(vec![
                0x00, 0x08, 0x8B, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x56,
            ]),
            Ok(vec![0x00, 0x08, 0x84, 0x23, 0x10, 0x00, 0x10, 0x01, 0x77]),
        ]);
        let iqos = Iqos::new(transport);

        let settings =
            block_on(iqos.read_vibration_settings(DeviceModel::Iluma)).expect("holder vibration");

        assert_eq!(settings, VibrationSettings::with_charge_start(false, true, true, false, true));
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[
                protocol::LOAD_VIBRATE_CHARGE_START_COMMAND.to_vec(),
                protocol::LOAD_VIBRATION_SETTINGS_COMMAND.to_vec(),
            ],
        );
    }

    #[test]
    fn update_vibration_settings_sends_expected_basic_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.update_vibration_settings(
            DeviceModel::IlumaOne,
            VibrationSettings::new(true, false, true, false),
        ))
        .expect("basic vibration update should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[vec![0x00, 0xC9, 0x44, 0x23, 0x10, 0x00, 0x01, 0x01, 0x65]],
        );
    }

    #[test]
    fn update_vibration_settings_preserves_holder_charge_start_when_unspecified() {
        let transport = MockTransport::with_responses([Ok(vec![
            0x00, 0x08, 0x8B, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x56,
        ])]);
        let iqos = Iqos::new(transport);

        block_on(iqos.update_vibration_settings(
            DeviceModel::Iluma,
            VibrationSettings::new(true, false, false, false),
        ))
        .expect("holder update should reuse current charge-start value");

        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[protocol::LOAD_VIBRATE_CHARGE_START_COMMAND.to_vec()],
        );
        assert_eq!(iqos.transport().recorded_sends().len(), 8);
    }

    #[test]
    fn update_vibration_settings_propagates_send_errors() {
        let transport = MockTransport::with_responses([])
            .with_send_results([Err(Error::Transport("send failed".to_string()))]);
        let iqos = Iqos::new(transport);

        let error = block_on(iqos.update_vibration_settings(
            DeviceModel::IlumaOne,
            VibrationSettings::new(true, false, true, false),
        ))
        .expect_err("send failures should surface");

        assert!(matches!(error, Error::Transport(message) if message == "send failed"));
    }

    #[test]
    fn set_brightness_high_sends_expected_command_sequence() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_brightness(BrightnessLevel::High))
            .expect("set high brightness should succeed");

        let expected: Vec<Vec<u8>> =
            BrightnessLevel::High.write_commands().iter().map(|cmd| cmd.to_vec()).collect();
        assert_eq!(iqos.transport().recorded_sends().as_slice(), expected.as_slice());
    }

    #[test]
    fn set_brightness_low_sends_expected_command_sequence() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_brightness(BrightnessLevel::Low))
            .expect("set low brightness should succeed");

        let expected: Vec<Vec<u8>> =
            BrightnessLevel::Low.write_commands().iter().map(|cmd| cmd.to_vec()).collect();
        assert_eq!(iqos.transport().recorded_sends().as_slice(), expected.as_slice());
    }

    #[test]
    fn set_brightness_propagates_send_errors() {
        let transport = MockTransport::with_responses([])
            .with_send_results([Err(Error::Transport("send failed".to_string()))]);
        let iqos = Iqos::new(transport);

        let error = block_on(iqos.set_brightness(BrightnessLevel::High))
            .expect_err("send errors should surface");

        assert!(matches!(error, Error::Transport(message) if message == "send failed"));
    }

    #[test]
    fn read_flexpuff_uses_expected_request_and_parses_response() {
        let transport = MockTransport::with_responses([Ok(vec![
            0x00, 0x90, 0x85, 0x22, 0x03, 0x01, 0x00, 0x00, 0x00,
        ])]);
        let iqos = Iqos::new(transport);

        let setting = block_on(iqos.read_flexpuff()).expect("flexpuff should parse");

        assert_eq!(setting, FlexPuffSetting::new(true));
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[protocol::LOAD_FLEXPUFF_COMMAND.to_vec()],
        );
    }

    #[test]
    fn set_flexpuff_sends_enable_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_flexpuff(FlexPuffSetting::new(true)))
            .expect("set flexpuff enabled should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[FlexPuffSetting::new(true).write_command().to_vec()],
        );
    }

    #[test]
    fn set_flexpuff_sends_disable_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_flexpuff(FlexPuffSetting::new(false)))
            .expect("set flexpuff disabled should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[FlexPuffSetting::new(false).write_command().to_vec()],
        );
    }

    #[test]
    fn read_flexpuff_propagates_transport_errors() {
        let iqos = Iqos::new(MockTransport::with_responses([Err(Error::Transport(
            "ble error".to_string(),
        ))]));

        let error = block_on(iqos.read_flexpuff()).expect_err("transport error should surface");

        assert!(matches!(error, Error::Transport(_)));
    }

    #[test]
    fn read_flexbattery_sends_both_load_requests_and_parses() {
        let transport = MockTransport::with_responses([
            Ok(vec![0x00, 0x08, 0x84, 0x25, 0x01, 0x00, 0x00, 0x00, 0x00]),
            Ok(vec![0x00, 0x08, 0x87, 0x24, 0x02, 0x00, 0x00, 0x00, 0x00]),
        ]);
        let iqos = Iqos::new(transport);

        let settings =
            block_on(iqos.read_flexbattery(DeviceModel::IlumaI)).expect("flexbattery should parse");

        assert_eq!(settings, FlexBatterySettings::new(FlexBatteryMode::Eco, Some(false)));
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[
                protocol::LOAD_FLEXBATTERY_COMMAND.to_vec(),
                protocol::LOAD_PAUSEMODE_COMMAND.to_vec(),
            ],
        );
    }

    #[test]
    fn read_flexbattery_rejects_unsupported_model() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        let error = block_on(iqos.read_flexbattery(DeviceModel::Iluma))
            .expect_err("unsupported model should fail");

        assert!(matches!(error, Error::Unsupported(_)));
    }

    #[test]
    fn set_flexbattery_eco_sends_expected_sequence() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_flexbattery(
            DeviceModel::IlumaI,
            FlexBatterySettings::new(FlexBatteryMode::Eco, None),
        ))
        .expect("set flexbattery eco should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[
                FlexBatteryMode::Eco.write_command().to_vec(),
                protocol::LOAD_FLEXBATTERY_COMMAND.to_vec(),
            ],
        );
    }

    #[test]
    fn set_flexbattery_with_pausemode_sends_full_sequence() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_flexbattery(
            DeviceModel::IlumaI,
            FlexBatterySettings::new(FlexBatteryMode::Performance, Some(true)),
        ))
        .expect("set flexbattery with pausemode should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[
                FlexBatteryMode::Performance.write_command().to_vec(),
                protocol::LOAD_FLEXBATTERY_COMMAND.to_vec(),
                protocol::pausemode_command(true).to_vec(),
                protocol::LOAD_PAUSEMODE_COMMAND.to_vec(),
            ],
        );
    }

    #[test]
    fn set_flexbattery_rejects_unsupported_model() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        let error = block_on(iqos.set_flexbattery(
            DeviceModel::Iluma,
            FlexBatterySettings::new(FlexBatteryMode::Eco, None),
        ))
        .expect_err("unsupported model should fail");

        assert!(matches!(error, Error::Unsupported(_)));
    }

    #[test]
    fn read_diagnosis_sends_all_commands_and_accumulates_responses() {
        // Four responses: telemetry(skipped-bad marker), timestamp, telemetry(skipped), battery
        let transport = MockTransport::with_responses([
            // Unknown header — should be silently skipped
            Ok(vec![0x00, 0x08, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00]),
            // Timestamp: days_used = 20 (0x0014)
            Ok(vec![0x00, 0x08, 0x80, 0x02, 0x14, 0x00, 0x00, 0x00]),
            // Unknown header — should be silently skipped
            Ok(vec![0x00, 0x08, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00]),
            // Battery: 4200 mV (0x1068)
            Ok(vec![0x00, 0x08, 0x88, 0x21, 0x00, 0x68, 0x10, 0x00, 0x00]),
        ]);
        let iqos = Iqos::new(transport);

        let data = block_on(iqos.read_diagnosis()).expect("diagnosis should succeed");

        assert_eq!(data.days_used, Some(20));
        assert_eq!(data.battery_voltage, Some(4200_f32 / 1000.0));
        assert_eq!(data.total_smoking_count, None);
        assert_eq!(
            iqos.transport().recorded_requests().len(),
            protocol::ALL_DIAGNOSIS_COMMANDS.len(),
        );
    }

    #[test]
    fn read_diagnosis_propagates_transport_errors() {
        let iqos = Iqos::new(MockTransport::with_responses([Err(Error::Transport(
            "ble error".to_string(),
        ))]));

        let error = block_on(iqos.read_diagnosis()).expect_err("transport error should surface");

        assert!(matches!(error, Error::Transport(_)));
    }

    #[test]
    fn lock_sends_expected_command_sequence() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.lock(DeviceModel::Iluma)).expect("lock should succeed");

        let expected: Vec<Vec<u8>> =
            protocol::lock_commands().iter().map(|cmd| cmd.to_vec()).collect();
        assert_eq!(iqos.transport().recorded_sends().as_slice(), expected.as_slice());
    }

    #[test]
    fn unlock_sends_expected_command_sequence() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.unlock(DeviceModel::IlumaOne)).expect("unlock should succeed");

        let expected: Vec<Vec<u8>> =
            protocol::unlock_commands().iter().map(|cmd| cmd.to_vec()).collect();
        assert_eq!(iqos.transport().recorded_sends().as_slice(), expected.as_slice());
    }

    #[test]
    fn lock_rejects_unknown_model() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        let error =
            block_on(iqos.lock(DeviceModel::Unknown)).expect_err("unknown model should fail");

        assert!(matches!(error, Error::Unsupported(_)));
    }

    #[test]
    fn unlock_propagates_send_errors() {
        let transport = MockTransport::with_responses([])
            .with_send_results([Err(Error::Transport("send failed".to_string()))]);
        let iqos = Iqos::new(transport);

        let error =
            block_on(iqos.unlock(DeviceModel::Iluma)).expect_err("send failure should surface");

        assert!(matches!(error, Error::Transport(message) if message == "send failed"));
    }

    #[test]
    fn set_smartgesture_enable_sends_correct_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_smartgesture(DeviceModel::Iluma, true))
            .expect("set smartgesture enabled should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[protocol::smartgesture_command(true).to_vec()],
        );
    }

    #[test]
    fn set_smartgesture_disable_sends_correct_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_smartgesture(DeviceModel::IlumaI, false))
            .expect("set smartgesture disabled should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[protocol::smartgesture_command(false).to_vec()],
        );
    }

    #[test]
    fn set_smartgesture_rejects_unsupported_model() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        let error = block_on(iqos.set_smartgesture(DeviceModel::IlumaOne, true))
            .expect_err("unsupported model should return error");

        assert!(matches!(error, Error::Unsupported(_)));
    }

    #[test]
    fn set_autostart_enable_sends_correct_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_autostart(DeviceModel::Iluma, true))
            .expect("set autostart enabled should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[protocol::autostart_command(true).to_vec()],
        );
    }

    #[test]
    fn set_autostart_disable_sends_correct_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.set_autostart(DeviceModel::IlumaI, false))
            .expect("set autostart disabled should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[protocol::autostart_command(false).to_vec()],
        );
    }

    #[test]
    fn set_autostart_rejects_unsupported_model() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        let error = block_on(iqos.set_autostart(DeviceModel::IlumaOne, true))
            .expect_err("unsupported model should return error");

        assert!(matches!(error, Error::Unsupported(_)));
    }
}
