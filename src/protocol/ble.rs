use uuid::{Uuid, uuid};

/// Standard Device Information service UUID.
pub const DEVICE_INFO_SERVICE_UUID: Uuid = uuid!("0000180a-0000-1000-8000-00805f9b34fb");

/// IQOS core control service UUID.
pub const IQOS_CORE_SERVICE_UUID: Uuid = uuid!("daebb240-b041-11e4-9e45-0002a5d5c51b");

/// Battery characteristic UUID exposed by the IQOS core service.
pub const BATTERY_CHARACTERISTIC_UUID: Uuid = uuid!("f8a54120-b041-11e4-9be7-0002a5d5c51b");

/// SCP control characteristic UUID used for request/response commands.
pub const SCP_CONTROL_CHARACTERISTIC_UUID: Uuid = uuid!("e16c6e20-b041-11e4-a4c3-0002a5d5c51b");

/// Standard GATT model number characteristic short UUID.
pub const MODEL_NUMBER_CHAR_UUID_PREFIX: &str = "00002a24";

/// Standard GATT serial number characteristic short UUID.
pub const SERIAL_NUMBER_CHAR_UUID_PREFIX: &str = "00002a25";

/// Standard GATT software revision characteristic short UUID.
pub const SOFTWARE_REVISION_CHAR_UUID_PREFIX: &str = "00002a28";

/// Standard GATT manufacturer name characteristic short UUID.
pub const MANUFACTURER_NAME_CHAR_UUID_PREFIX: &str = "00002a29";
