use iqos::{DeviceCapability, DeviceStatus, Iqos, IqosBle};

use crate::{TestResult, exercises};

pub(crate) async fn snapshot(session: &IqosBle, iqos: &Iqos<IqosBle>) -> TestResult {
    let model = session.model();

    match iqos.read_device_status(model, session.device_info().clone()).await {
        Ok(status) => print_status_snapshot(&status),
        Err(e) => {
            let info = session.device_info();
            println!("  Model:          {:?}", model);
            println!(
                "  Model Number:   {}",
                clean(info.model_number.as_deref().unwrap_or("(missing)"))
            );
            println!(
                "  Serial Number:  {}",
                clean(info.serial_number.as_deref().unwrap_or("(missing)"))
            );
            println!(
                "  Manufacturer:   {}",
                clean(info.manufacturer_name.as_deref().unwrap_or("(missing)"))
            );
            println!(
                "  Software Rev:   {}",
                clean(info.software_revision.as_deref().unwrap_or("(missing)"))
            );
            println!("  Device Status:  (read failed: {e})");
        }
    }

    match session.read_battery_level().await {
        Ok(level) => println!("  Battery:  {level}%"),
        Err(e) => println!("  Battery:  (read failed: {e})"),
    }

    match iqos.read_brightness().await {
        Ok(b) => println!("  Bright:   {b}"),
        Err(e) => println!("  Bright:   (read failed: {e})"),
    }

    match iqos.read_vibration_settings(model).await {
        Ok(v) => {
            let cs =
                v.when_charging_start().map(|b| format!(" | charge_start={b}")).unwrap_or_default();
            println!(
                "  Vibe:     heating={} use={} puff_end={} terminated={}{}",
                v.when_heating_start(),
                v.when_starting_to_use(),
                v.when_puff_end(),
                v.when_manually_terminated(),
                cs,
            );
        }
        Err(e) => println!("  Vibe:     (read failed: {e})"),
    }

    if model.supports(DeviceCapability::FlexPuff) {
        match iqos.read_flexpuff(model).await {
            Ok(fp) => {
                println!("  FlexPuff: {}", if fp.is_enabled() { "enabled" } else { "disabled" })
            }
            Err(e) => println!("  FlexPuff: (read failed: {e})"),
        }
    }

    if model.supports(DeviceCapability::FlexBattery) {
        match iqos.read_flexbattery(model).await {
            Ok(fb) => println!("  FlexBatt: {:?} | pause={:?}", fb.mode(), fb.pause_mode()),
            Err(e) => println!("  FlexBatt: (read failed: {e})"),
        }
    }

    Ok(())
}

pub(crate) async fn exercise_all(
    session: &IqosBle,
    iqos: &Iqos<IqosBle>,
    vibrate_millis: u64,
) -> TestResult {
    let model = session.model();
    let mut failures: u32 = 0;

    macro_rules! exercise {
        ($name:literal, $call:expr) => {{
            print!("  [{}] ", $name);
            match $call {
                Ok(_) => println!("passed"),
                Err(ref e) => {
                    println!("FAILED: {e}");
                    failures += 1;
                }
            }
        }};
    }

    exercise!("brightness", exercises::brightness(iqos).await);
    exercise!("vibration settings", exercises::vibration_settings(iqos, model).await);
    exercise!("direct vibration", exercises::direct_vibration(iqos, vibrate_millis).await);
    exercise!("lock / unlock", exercises::lock_unlock(iqos, model).await);

    if model.supports(DeviceCapability::FlexPuff) {
        exercise!("flexpuff", exercises::flexpuff(iqos, model).await);
    }
    if model.supports(DeviceCapability::FlexBattery) {
        exercise!("flexbattery", exercises::flexbattery(iqos, model).await);
    }
    if model.supports(DeviceCapability::SmartGesture) {
        exercise!("smart gesture", exercises::smartgesture(iqos, model).await);
    }
    if model.supports(DeviceCapability::AutoStart) {
        exercise!("auto start", exercises::autostart(iqos, model).await);
    }

    if failures > 0 { Err(format!("{failures} exercise(s) failed").into()) } else { Ok(()) }
}

fn clean(s: &str) -> &str {
    s.trim_end_matches('\0')
}

fn print_status_snapshot(status: &DeviceStatus) {
    for line in status_snapshot_lines(status) {
        println!("{line}");
    }
}

fn status_snapshot_lines(status: &DeviceStatus) -> Vec<String> {
    let info = &status.device_info;
    let missing = "(missing)";
    let mut lines = vec![
        format!("  Model:          {:?}", status.model),
        format!("  Model Number:   {}", clean(info.model_number.as_deref().unwrap_or(missing))),
        format!("  Serial Number:  {}", clean(info.serial_number.as_deref().unwrap_or(missing))),
        format!(
            "  Manufacturer:   {}",
            clean(info.manufacturer_name.as_deref().unwrap_or(missing))
        ),
        format!(
            "  Software Rev:   {}",
            clean(info.software_revision.as_deref().unwrap_or(missing))
        ),
    ];

    if status.model.supports_holder_features() {
        let holder_firmware = status
            .holder_firmware
            .map(|firmware| firmware.to_string())
            .unwrap_or_else(|| missing.to_string());
        lines.extend([
            format!("  Stick Product:  {}", status.product_number),
            format!("  Stick Firmware: {}", status.stick_firmware),
            format!(
                "  Holder Product: {}",
                status.holder_product_number.as_deref().unwrap_or(missing)
            ),
            format!("  Holder FW:      {}", holder_firmware),
        ]);
    } else {
        lines.extend([
            format!("  Product Number: {}", status.product_number),
            format!("  Firmware:       {}", status.stick_firmware),
        ]);
    }

    match status.battery_voltage {
        Some(voltage) => lines.push(format!("  Battery V:      {voltage:.3} V")),
        None => lines.push("  Battery V:      read failed".to_string()),
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::status_snapshot_lines;
    use iqos::{DeviceInfo, DeviceModel, DeviceStatus, FirmwareVersion};

    #[test]
    fn status_snapshot_lines_include_required_one_piece_status_fields() {
        let status = DeviceStatus {
            model: DeviceModel::IlumaOne,
            device_info: DeviceInfo {
                model_number: Some("HMIR-ONE".to_string()),
                serial_number: Some("SERIAL-ONE".to_string()),
                software_revision: Some("SW-1".to_string()),
                manufacturer_name: Some("PMI".to_string()),
            },
            product_number: "STICK-PRODUCT".to_string(),
            stick_firmware: FirmwareVersion { major: 1, minor: 2, patch: 3, year: 24 },
            holder_product_number: None,
            holder_firmware: None,
            battery_voltage: Some(4.123),
        };

        let lines = status_snapshot_lines(&status);

        assert!(lines.contains(&"  Model:          IlumaOne".to_string()));
        assert!(lines.contains(&"  Model Number:   HMIR-ONE".to_string()));
        assert!(lines.contains(&"  Serial Number:  SERIAL-ONE".to_string()));
        assert!(lines.contains(&"  Manufacturer:   PMI".to_string()));
        assert!(lines.contains(&"  Software Rev:   SW-1".to_string()));
        assert!(lines.contains(&"  Product Number: STICK-PRODUCT".to_string()));
        assert!(lines.contains(&"  Firmware:       v1.2.3.24".to_string()));
    }

    #[test]
    fn status_snapshot_lines_include_holder_product_number_and_firmware() {
        let status = DeviceStatus {
            model: DeviceModel::IlumaIPrime,
            device_info: DeviceInfo {
                model_number: Some("HMIR-PRIME".to_string()),
                serial_number: Some("SERIAL-PRIME".to_string()),
                software_revision: Some("SW-2".to_string()),
                manufacturer_name: Some("PMI".to_string()),
            },
            product_number: "STICK-PRODUCT".to_string(),
            stick_firmware: FirmwareVersion { major: 2, minor: 3, patch: 4, year: 25 },
            holder_product_number: Some("HOLDER-PRODUCT".to_string()),
            holder_firmware: Some(FirmwareVersion { major: 5, minor: 6, patch: 7, year: 26 }),
            battery_voltage: None,
        };

        let lines = status_snapshot_lines(&status);

        assert!(lines.contains(&"  Stick Product:  STICK-PRODUCT".to_string()));
        assert!(lines.contains(&"  Stick Firmware: v2.3.4.25".to_string()));
        assert!(lines.contains(&"  Holder Product: HOLDER-PRODUCT".to_string()));
        assert!(lines.contains(&"  Holder FW:      v5.6.7.26".to_string()));
    }
}
