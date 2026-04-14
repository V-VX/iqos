//! BLE-focused IQOS session and metadata types.
//!
//! This module contains the first extracted BLE-specific core logic from the
//! legacy `iqos_cli` implementation: device-model detection, device information
//! loading, characteristic discovery, and the low-level request/response path
//! built around the SCP control characteristic.

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use futures::StreamExt;

use crate::{
    Error, Result,
    protocol::{
        BATTERY_CHARACTERISTIC_UUID, DEVICE_INFO_SERVICE_UUID, DeviceInfo, DeviceModel,
        MANUFACTURER_NAME_CHAR_UUID_PREFIX, MODEL_NUMBER_CHAR_UUID_PREFIX,
        SCP_CONTROL_CHARACTERISTIC_UUID, SERIAL_NUMBER_CHAR_UUID_PREFIX,
        SOFTWARE_REVISION_CHAR_UUID_PREFIX,
    },
};

/// BLE session around a connected IQOS peripheral.
#[derive(Debug, Clone)]
pub struct IqosBle {
    peripheral: Peripheral,
    battery_characteristic: Characteristic,
    scp_control_characteristic: Characteristic,
    model: DeviceModel,
    device_info: DeviceInfo,
}

impl IqosBle {
    /// Connect to a peripheral, discover services, subscribe to the control
    /// channel, and load basic device metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if connection, service discovery, subscription, or
    /// metadata loading fails.
    pub async fn connect_and_discover(peripheral: Peripheral) -> Result<Self> {
        if !peripheral.is_connected().await.map_err(|error| Error::Transport(error.to_string()))? {
            peripheral.connect().await.map_err(|error| Error::Transport(error.to_string()))?;
        }

        peripheral
            .discover_services()
            .await
            .map_err(|error| Error::Transport(error.to_string()))?;

        let battery_characteristic = find_characteristic(&peripheral, BATTERY_CHARACTERISTIC_UUID)?;
        let scp_control_characteristic =
            find_characteristic(&peripheral, SCP_CONTROL_CHARACTERISTIC_UUID)?;
        let model = detect_model(&peripheral).await?;
        let device_info = load_device_info(&peripheral).await?;

        peripheral
            .subscribe(&scp_control_characteristic)
            .await
            .map_err(|error| Error::Transport(error.to_string()))?;

        Ok(Self {
            peripheral,
            battery_characteristic,
            scp_control_characteristic,
            model,
            device_info,
        })
    }

    /// Borrow the detected device model.
    #[must_use]
    pub const fn model(&self) -> DeviceModel {
        self.model
    }

    /// Borrow the loaded device information snapshot.
    #[must_use]
    pub const fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    /// Read the raw battery status frame from the device.
    ///
    /// # Errors
    ///
    /// Returns an error if the BLE read fails.
    pub async fn read_battery_frame(&self) -> Result<Vec<u8>> {
        self.peripheral
            .read(&self.battery_characteristic)
            .await
            .map_err(|error| Error::Transport(error.to_string()))
    }

    /// Send a command over the SCP control characteristic.
    ///
    /// # Errors
    ///
    /// Returns an error if the BLE write fails.
    pub async fn send(&self, command: &[u8]) -> Result<()> {
        self.peripheral
            .write(&self.scp_control_characteristic, command, WriteType::WithResponse)
            .await
            .map_err(|error| Error::Transport(error.to_string()))
    }

    /// Send a command and wait for the next notification frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails, notifications cannot be opened, or
    /// no response frame arrives.
    pub async fn request(&self, command: &[u8]) -> Result<Vec<u8>> {
        self.send(command).await?;
        let mut notifications = self
            .peripheral
            .notifications()
            .await
            .map_err(|error| Error::Transport(error.to_string()))?;

        notifications
            .next()
            .await
            .map(|notification| notification.value)
            .ok_or_else(|| Error::Transport("no BLE response notification received".to_string()))
    }
}

async fn detect_model(peripheral: &Peripheral) -> Result<DeviceModel> {
    let properties = peripheral
        .properties()
        .await
        .map_err(|error| Error::Transport(error.to_string()))?
        .ok_or_else(|| Error::Transport("missing BLE properties".to_string()))?;

    Ok(properties.local_name.as_deref().map_or(DeviceModel::Unknown, DeviceModel::from_local_name))
}

async fn load_device_info(peripheral: &Peripheral) -> Result<DeviceInfo> {
    let service = peripheral
        .services()
        .into_iter()
        .find(|service| service.uuid == DEVICE_INFO_SERVICE_UUID)
        .ok_or_else(|| Error::Transport("device information service not found".to_string()))?;

    let mut info = DeviceInfo::default();

    for characteristic in &service.characteristics {
        let uuid_prefix = characteristic.uuid.to_string();
        let Some(prefix) = uuid_prefix.split('-').next() else {
            continue;
        };

        let value = peripheral
            .read(characteristic)
            .await
            .map_err(|error| Error::Transport(error.to_string()))?;
        let value = String::from_utf8_lossy(&value).to_string();

        match prefix {
            MODEL_NUMBER_CHAR_UUID_PREFIX => info.model_number = Some(value),
            SERIAL_NUMBER_CHAR_UUID_PREFIX => info.serial_number = Some(value),
            SOFTWARE_REVISION_CHAR_UUID_PREFIX => info.software_revision = Some(value),
            MANUFACTURER_NAME_CHAR_UUID_PREFIX => info.manufacturer_name = Some(value),
            _ => {}
        }
    }

    Ok(info)
}

fn find_characteristic(peripheral: &Peripheral, target_uuid: uuid::Uuid) -> Result<Characteristic> {
    peripheral
        .services()
        .into_iter()
        .flat_map(|service| service.characteristics.into_iter())
        .find(|characteristic| characteristic.uuid == target_uuid)
        .ok_or_else(|| Error::Transport(format!("characteristic not found: {target_uuid}")))
}

#[cfg(test)]
mod tests {
    use crate::protocol::DeviceModel;

    #[test]
    fn classifies_iluma_i_prime_before_generic_matches() {
        assert_eq!(DeviceModel::from_local_name("IQOS ILUMA i PRIME"), DeviceModel::IlumaIPrime,);
    }

    #[test]
    fn classifies_iluma_i_one_before_generic_one_matches() {
        assert_eq!(DeviceModel::from_local_name("IQOS ILUMA i ONE"), DeviceModel::IlumaIOne,);
    }

    #[test]
    fn classifies_unknown_names() {
        assert_eq!(DeviceModel::from_local_name("mystery device"), DeviceModel::Unknown);
    }
}
