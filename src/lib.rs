//! # iqos
//!
//! Rust library for controlling IQOS devices over BLE, exposing diagnostic
//! telemetry and device controls not available through the official IQOS app.
//!
//! ## Layers
//!
//! 1. [`protocol`] — command builders, response parsers, and typed domain values
//! 2. [`transport`] — transport contract shared by BLE and future USB backends
//! 3. [`transports`] — backend implementations (BLE via btleplug, USB reserved)
//!
//! ## Features
//!
//! - `btleplug-support`: enables the BLE backend via btleplug
//! - `usb-support`: reserved for future USB transport (not yet implemented)

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
    BrightnessLevel, DeviceCapability, DeviceInfo, DeviceModel, DeviceStatus, DiagnosticData,
    FirmwareKind, FirmwareVersion, FlexBatteryMode, FlexBatterySettings, FlexPuffSetting,
    VibrationSettings,
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

    /// Read the current Auto Start setting for the provided model.
    ///
    /// Auto Start is only supported on holder-based ILUMA and ILUMA i models.
    ///
    /// # Errors
    ///
    /// Returns an error if the model does not support Auto Start, a transport
    /// request fails, or the response frame cannot be decoded.
    pub async fn read_autostart(&self, model: DeviceModel) -> Result<bool> {
        if !model.supports(DeviceCapability::AutoStart) {
            return Err(Error::Unsupported(format!(
                "Auto Start is not supported for model {model:?}"
            )));
        }
        let response = self.transport.request(&protocol::LOAD_AUTOSTART_COMMAND).await?;
        protocol::autostart_from_response(&response)
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

    /// Read the current battery voltage via the SCP diagnostic command.
    ///
    /// Returns the raw cell voltage in volts (e.g. `4.2`). This uses the
    /// request/response path and is suitable for on-demand refreshes after the
    /// BLE session is established.
    ///
    /// For the initial connection snapshot (battery percentage via GATT direct
    /// read), use [`IqosBle::read_battery_level`](crate::ble::IqosBle::read_battery_level).
    ///
    /// # Errors
    ///
    /// Returns an error if the transport request fails, the response cannot be
    /// decoded, or the battery voltage field is absent from the frame.
    pub async fn read_battery_voltage(&self) -> Result<f32> {
        let response = self.transport.request(&protocol::LOAD_BATTERY_VOLTAGE_COMMAND).await?;
        protocol::DiagnosticDataBuilder::default()
            .parse(&response)?
            .build()
            .battery_voltage
            .ok_or_else(|| {
                Error::ProtocolDecode("battery voltage not present in response".to_string())
            })
    }

    /// Read a device status snapshot: stick firmware, holder firmware (folder-type only),
    /// and battery voltage.
    ///
    /// Firmware reads are fatal — they propagate errors. Battery voltage uses `.ok()` so a
    /// failed diagnostic read yields `None` rather than aborting the whole status read.
    ///
    /// # Errors
    ///
    /// Returns an error if either firmware request fails or the response cannot be decoded.
    pub async fn read_device_status(&self, model: DeviceModel) -> Result<DeviceStatus> {
        let stick_firmware = self.read_firmware_version(FirmwareKind::Stick).await?;
        let holder_firmware = if model.supports_holder_features() {
            Some(self.read_firmware_version(FirmwareKind::Holder).await?)
        } else {
            None
        };
        let battery_voltage = self.read_battery_voltage().await.ok();
        Ok(DeviceStatus { stick_firmware, holder_firmware, battery_voltage })
    }

    /// Trigger an immediate vibration burst on the device.
    ///
    /// Used by [`find_my_iqos_start`](Self::find_my_iqos_start). Can also be
    /// called directly when raw vibration control is needed. Stop the burst
    /// with [`vibrate_stop`](Self::vibrate_stop).
    ///
    /// # Errors
    ///
    /// Returns an error if the transport send fails.
    pub async fn vibrate_start(&self) -> Result<()> {
        self.transport.send(&protocol::START_VIBRATE_COMMAND).await
    }

    /// Stop an ongoing vibration burst on the device.
    ///
    /// Counterpart to [`vibrate_start`](Self::vibrate_start) and
    /// [`find_my_iqos_start`](Self::find_my_iqos_start).
    ///
    /// # Errors
    ///
    /// Returns an error if the transport send fails.
    pub async fn vibrate_stop(&self) -> Result<()> {
        self.transport.send(&protocol::STOP_VIBRATE_COMMAND).await
    }

    /// Start a Find My IQOS session — begin vibrating the device so it can be
    /// physically located.
    ///
    /// The device continues to vibrate until [`find_my_iqos_stop`](Self::find_my_iqos_stop)
    /// is called. Delegates to [`vibrate_start`](Self::vibrate_start).
    ///
    /// # Errors
    ///
    /// Returns an error if the transport send fails.
    pub async fn find_my_iqos_start(&self) -> Result<()> {
        self.vibrate_start().await
    }

    /// Stop a Find My IQOS session — halt the vibration burst.
    ///
    /// Call after [`find_my_iqos_start`](Self::find_my_iqos_start) once the
    /// device has been located. Delegates to [`vibrate_stop`](Self::vibrate_stop).
    ///
    /// # Errors
    ///
    /// Returns an error if the transport send fails.
    pub async fn find_my_iqos_stop(&self) -> Result<()> {
        self.vibrate_stop().await
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

    #[test]
    fn read_autostart_uses_expected_request_and_parses_response() {
        let transport = MockTransport::with_responses([Ok(vec![
            0x00, 0x08, 0x87, 0x24, 0x01, 0x01, 0x00, 0x00, 0xA5,
        ])]);
        let iqos = Iqos::new(transport);

        let enabled =
            block_on(iqos.read_autostart(DeviceModel::Iluma)).expect("autostart should parse");

        assert!(enabled);
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[protocol::LOAD_AUTOSTART_COMMAND.to_vec()],
        );
    }

    #[test]
    fn read_autostart_rejects_unsupported_model() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        let error = block_on(iqos.read_autostart(DeviceModel::IlumaOne))
            .expect_err("unsupported model should return error");

        assert!(matches!(error, Error::Unsupported(_)));
    }

    #[test]
    fn read_autostart_propagates_transport_errors() {
        let iqos = Iqos::new(MockTransport::with_responses([Err(Error::Transport(
            "transport down".to_string(),
        ))]));

        let error = block_on(iqos.read_autostart(DeviceModel::Iluma))
            .expect_err("transport error should surface");

        assert!(matches!(error, Error::Transport(message) if message == "transport down"));
    }

    #[test]
    fn read_battery_voltage_parses_response() {
        // battery voltage frame: header [0x88, 0x21] at bytes[2..4], raw mV at bytes[5..7]
        // 0xA8=168, 0x10=16 → LE u16 = 0x10A8 = 4264 → 4.264 V
        let transport = MockTransport::with_responses([Ok(vec![
            0x00, 0x08, 0x88, 0x21, 0x00, 0xA8, 0x10, 0x00, 0x00,
        ])]);
        let iqos = Iqos::new(transport);

        let voltage = block_on(iqos.read_battery_voltage()).expect("voltage should parse");

        assert!((voltage - 4264_f32 / 1000.0).abs() < f32::EPSILON);
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[protocol::LOAD_BATTERY_VOLTAGE_COMMAND.to_vec()],
        );
    }

    #[test]
    fn vibrate_start_sends_expected_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.vibrate_start()).expect("vibrate start should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[protocol::START_VIBRATE_COMMAND.to_vec()],
        );
    }

    #[test]
    fn vibrate_stop_sends_expected_command() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.vibrate_stop()).expect("vibrate stop should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[protocol::STOP_VIBRATE_COMMAND.to_vec()],
        );
    }

    #[test]
    fn find_my_iqos_start_delegates_to_vibrate_start() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.find_my_iqos_start()).expect("find my iqos start should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[protocol::START_VIBRATE_COMMAND.to_vec()],
        );
    }

    #[test]
    fn find_my_iqos_stop_delegates_to_vibrate_stop() {
        let iqos = Iqos::new(MockTransport::with_responses([]));

        block_on(iqos.find_my_iqos_stop()).expect("find my iqos stop should succeed");

        assert_eq!(
            iqos.transport().recorded_sends().as_slice(),
            &[protocol::STOP_VIBRATE_COMMAND.to_vec()],
        );
    }

    #[test]
    fn read_device_status_for_one_piece_model_sends_stick_firmware_and_battery() {
        // stick firmware response, then battery voltage response
        let transport = MockTransport::with_responses([
            Ok(vec![0x00, 0xC0, 0x88, 0x00, 0x00, 0x00, 0x02, 0x05, 0x07, 0x18]),
            Ok(vec![0x00, 0x08, 0x88, 0x21, 0x00, 0xA8, 0x10, 0x00, 0x00]),
        ]);
        let iqos = Iqos::new(transport);

        let status = block_on(iqos.read_device_status(DeviceModel::IlumaOne))
            .expect("one-piece status should succeed");

        assert_eq!(
            status.stick_firmware,
            FirmwareVersion { major: 2, minor: 5, patch: 7, year: 24 }
        );
        assert!(status.holder_firmware.is_none());
        assert!(status.battery_voltage.is_some());
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[
                protocol::LOAD_STICK_FIRMWARE_VERSION_COMMAND.to_vec(),
                protocol::LOAD_BATTERY_VOLTAGE_COMMAND.to_vec(),
            ],
        );
    }

    #[test]
    fn read_device_status_for_folder_model_sends_both_firmware_commands() {
        // stick firmware, holder firmware, battery voltage
        let transport = MockTransport::with_responses([
            Ok(vec![0x00, 0xC0, 0x88, 0x00, 0x00, 0x00, 0x02, 0x05, 0x07, 0x18]),
            Ok(vec![0x00, 0x08, 0x88, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x19]),
            Ok(vec![0x00, 0x08, 0x88, 0x21, 0x00, 0xA8, 0x10, 0x00, 0x00]),
        ]);
        let iqos = Iqos::new(transport);

        let status = block_on(iqos.read_device_status(DeviceModel::Iluma))
            .expect("folder status should succeed");

        assert_eq!(
            status.stick_firmware,
            FirmwareVersion { major: 2, minor: 5, patch: 7, year: 24 }
        );
        assert_eq!(
            status.holder_firmware,
            Some(FirmwareVersion { major: 1, minor: 2, patch: 3, year: 25 }),
        );
        assert!(status.battery_voltage.is_some());
        assert_eq!(
            iqos.transport().recorded_requests().as_slice(),
            &[
                protocol::LOAD_STICK_FIRMWARE_VERSION_COMMAND.to_vec(),
                protocol::LOAD_HOLDER_FIRMWARE_VERSION_COMMAND.to_vec(),
                protocol::LOAD_BATTERY_VOLTAGE_COMMAND.to_vec(),
            ],
        );
    }

    #[test]
    fn read_device_status_battery_failure_yields_none_not_error() {
        // stick firmware ok, battery fails
        let transport = MockTransport::with_responses([
            Ok(vec![0x00, 0xC0, 0x88, 0x00, 0x00, 0x00, 0x02, 0x05, 0x07, 0x18]),
            Err(Error::Transport("battery read failed".to_string())),
        ]);
        let iqos = Iqos::new(transport);

        let status = block_on(iqos.read_device_status(DeviceModel::IlumaOne))
            .expect("battery failure should not abort status read");

        assert!(status.battery_voltage.is_none());
    }

    #[test]
    fn read_device_status_propagates_stick_firmware_error() {
        let iqos = Iqos::new(MockTransport::with_responses([Err(Error::Transport(
            "ble error".to_string(),
        ))]));

        let error = block_on(iqos.read_device_status(DeviceModel::IlumaOne))
            .expect_err("stick firmware error should surface");

        assert!(matches!(error, Error::Transport(_)));
    }
}
