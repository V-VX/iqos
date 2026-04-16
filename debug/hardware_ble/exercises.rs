use iqos::{
    BrightnessLevel, DeviceModel, FlexBatteryMode, FlexBatterySettings, FlexPuffSetting, Iqos,
    IqosBle, VibrationSettings,
};
use tokio::time::{Duration, sleep};

use crate::TestResult;

/// Run a fallible step; print `[✅]` on success or `[❌]` and return early on failure.
macro_rules! run {
    ($label:expr, $result:expr) => {{
        match $result {
            Ok(value) => {
                println!("    [✅] {}", $label);
                value
            }
            Err(error) => {
                println!("    [❌] {} — {}", $label, error);
                return Err(error.into());
            }
        }
    }};
}

pub(crate) async fn brightness(iqos: &Iqos<IqosBle>) -> TestResult {
    let original = run!("read brightness", iqos.read_brightness().await);
    let toggled = match original {
        BrightnessLevel::High => BrightnessLevel::Low,
        BrightnessLevel::Low => BrightnessLevel::High,
    };

    run!(format!("set brightness → {toggled}"), iqos.set_brightness(toggled).await);

    let read_back = run!("verify brightness", iqos.read_brightness().await);
    if read_back != toggled {
        let _ = iqos.set_brightness(original).await;
        return Err(format!("brightness mismatch: expected {toggled}, got {read_back}").into());
    }

    run!(format!("restore brightness → {original}"), iqos.set_brightness(original).await);

    let restored = run!("verify brightness restored", iqos.read_brightness().await);
    if restored != original {
        return Err(format!("restore failed: expected {original}, got {restored}").into());
    }

    Ok(())
}

pub(crate) async fn vibration_settings(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let original = run!("read vibration settings", iqos.read_vibration_settings(model).await);

    let toggled = if let Some(cs) = original.when_charging_start() {
        VibrationSettings::with_charge_start(
            !original.when_heating_start(),
            original.when_starting_to_use(),
            original.when_puff_end(),
            original.when_manually_terminated(),
            cs,
        )
    } else {
        VibrationSettings::new(
            !original.when_heating_start(),
            original.when_starting_to_use(),
            original.when_puff_end(),
            original.when_manually_terminated(),
        )
    };

    run!(
        "write vibration settings (toggle heating start)",
        iqos.update_vibration_settings(model, toggled).await
    );

    let read_back = run!("verify vibration settings", iqos.read_vibration_settings(model).await);
    if read_back.when_heating_start() != toggled.when_heating_start() {
        let _ = iqos.update_vibration_settings(model, original).await;
        return Err(format!(
            "heating_start mismatch: expected {}, got {}",
            toggled.when_heating_start(),
            read_back.when_heating_start(),
        )
        .into());
    }

    run!("restore vibration settings", iqos.update_vibration_settings(model, original).await);

    let restored = run!("verify vibration restored", iqos.read_vibration_settings(model).await);
    if restored.when_heating_start() != original.when_heating_start() {
        return Err(format!(
            "restore failed: expected heating_start={}",
            original.when_heating_start()
        )
        .into());
    }

    Ok(())
}

pub(crate) async fn direct_vibration(iqos: &Iqos<IqosBle>, millis: u64) -> TestResult {
    run!("vibrate start", iqos.vibrate_start().await);
    sleep(Duration::from_millis(millis)).await;
    run!("vibrate stop", iqos.vibrate_stop().await);
    Ok(())
}

pub(crate) async fn lock_unlock(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    run!("lock", iqos.lock(model).await);
    run!("unlock", iqos.unlock(model).await);
    Ok(())
}

pub(crate) async fn flexpuff(iqos: &Iqos<IqosBle>) -> TestResult {
    let original = run!("read flexpuff", iqos.read_flexpuff().await);
    let toggled = FlexPuffSetting::new(!original.is_enabled());
    let label = if toggled.is_enabled() { "enabled" } else { "disabled" };

    run!(format!("set flexpuff → {label}"), iqos.set_flexpuff(toggled).await);

    let read_back = run!("verify flexpuff", iqos.read_flexpuff().await);
    if read_back != toggled {
        let _ = iqos.set_flexpuff(original).await;
        return Err(
            format!("flexpuff mismatch: expected {:?}, got {:?}", toggled, read_back).into()
        );
    }

    let orig_label = if original.is_enabled() { "enabled" } else { "disabled" };
    run!(format!("restore flexpuff → {orig_label}"), iqos.set_flexpuff(original).await);

    let restored = run!("verify flexpuff restored", iqos.read_flexpuff().await);
    if restored != original {
        return Err(format!(
            "flexpuff restore failed: expected {:?}, got {:?}",
            original, restored
        )
        .into());
    }

    Ok(())
}

pub(crate) async fn flexbattery(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let original = run!("read flexbattery", iqos.read_flexbattery(model).await);
    let toggled_mode = match original.mode() {
        FlexBatteryMode::Performance => FlexBatteryMode::Eco,
        FlexBatteryMode::Eco => FlexBatteryMode::Performance,
    };
    let toggled = FlexBatterySettings::new(toggled_mode, original.pause_mode());

    run!(format!("set flexbattery → {toggled_mode:?}"), iqos.set_flexbattery(model, toggled).await);

    let read_back = run!("verify flexbattery", iqos.read_flexbattery(model).await);
    if read_back.mode() != toggled_mode {
        let _ = iqos.set_flexbattery(model, original).await;
        return Err(format!(
            "mode mismatch: expected {toggled_mode:?}, got {:?}",
            read_back.mode()
        )
        .into());
    }

    run!(
        format!("restore flexbattery → {:?}", original.mode()),
        iqos.set_flexbattery(model, original).await
    );

    let restored = run!("verify flexbattery restored", iqos.read_flexbattery(model).await);
    if restored.mode() != original.mode() {
        return Err(format!(
            "restore failed: expected {:?}, got {:?}",
            original.mode(),
            restored.mode()
        )
        .into());
    }

    Ok(())
}

pub(crate) async fn smartgesture(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    run!("enable smart gesture", iqos.set_smartgesture(model, true).await);
    run!("disable smart gesture", iqos.set_smartgesture(model, false).await);
    Ok(())
}

pub(crate) async fn autostart(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    run!("enable auto start", iqos.set_autostart(model, true).await);
    run!("disable auto start", iqos.set_autostart(model, false).await);
    Ok(())
}
