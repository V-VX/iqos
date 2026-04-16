use iqos::{DeviceCapability, FirmwareKind, Iqos, IqosBle};

use crate::{TestResult, exercises};

pub(crate) async fn snapshot(session: &IqosBle, iqos: &Iqos<IqosBle>) -> TestResult {
    let model = session.model();
    let info = session.device_info();

    println!("  Model:    {:?}", model);
    println!("  Serial:   {}", clean(info.serial_number.as_deref().unwrap_or("(none)")));
    println!("  SW rev:   {}", clean(info.software_revision.as_deref().unwrap_or("(none)")));
    println!("  Manuf:    {}", clean(info.manufacturer_name.as_deref().unwrap_or("(none)")));

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
        match iqos.read_flexpuff().await {
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

    match iqos.read_firmware_version(FirmwareKind::Stick).await {
        Ok(fw) => println!("  FW stick: {fw}"),
        Err(e) => println!("  FW stick: (read failed: {e})"),
    }

    if model.supports_holder_features() {
        match iqos.read_firmware_version(FirmwareKind::Holder).await {
            Ok(fw) => println!("  FW holder: {fw}"),
            Err(e) => println!("  FW holder: (read failed: {e})"),
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
        exercise!("flexpuff", exercises::flexpuff(iqos).await);
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
