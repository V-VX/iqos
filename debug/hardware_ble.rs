use std::{env, io};

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use iqos::{
    BrightnessLevel, DeviceCapability, DeviceInfo, DeviceModel, FirmwareKind, FlexBatteryMode,
    FlexBatterySettings, FlexPuffSetting, Iqos, IqosBle, VibrationSettings,
};
use tokio::time::{Duration, sleep};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Evaluate `$result`; on error print `[❌] $label: <error>` and return early.
macro_rules! run {
    ($label:expr, $result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                println!("[❌] {}: {e}", $label);
                return Err(e.into());
            }
        }
    };
}

// ── config ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct HardwareConfig {
    /// Optional substring to filter BLE device names.  `None` = first IQOS device found.
    name_filter: Option<String>,
    scan_seconds: u64,
    vibrate_millis: u64,
    allow_stateful_writes: bool,
}

impl HardwareConfig {
    fn from_env() -> TestResult<Self> {
        let name_filter = match env::var("IQOS_TEST_NAME_SUBSTRING") {
            Ok(value) if !value.trim().is_empty() => Some(value),
            _ => None,
        };

        let scan_seconds = match env::var("IQOS_TEST_SCAN_SECONDS") {
            Ok(value) => value.parse::<u64>().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidInput, format!("IQOS_TEST_SCAN_SECONDS: {e}"))
            })?,
            Err(_) => 5,
        };

        let vibrate_millis = match env::var("IQOS_TEST_VIBRATE_MILLIS") {
            Ok(value) => value.parse::<u64>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("IQOS_TEST_VIBRATE_MILLIS: {e}"),
                )
            })?,
            Err(_) => 750,
        };

        let allow_stateful_writes = env_flag("IQOS_TEST_ALLOW_STATEFUL_WRITES")?;

        Ok(Self { name_filter, scan_seconds, vibrate_millis, allow_stateful_writes })
    }
}

// ── entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let config = match HardwareConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            println!("[❌] {e}");
            std::process::exit(1);
        }
    };

    println!("=== IQOS Hardware Debug ===");
    match &config.name_filter {
        Some(f) => println!("filter:  {f:?}"),
        None => println!("filter:  (any IQOS device)"),
    }
    println!("stateful: {}", if config.allow_stateful_writes { "enabled" } else { "disabled" });

    let mut passed = 0usize;
    let mut failed = 0usize;

    run_suite("snapshot", suite_snapshot(&config), &mut passed, &mut failed).await;
    run_suite("read features", suite_read_features(&config), &mut passed, &mut failed).await;

    if config.allow_stateful_writes {
        run_suite("exercise all", suite_exercise_all(&config), &mut passed, &mut failed).await;
    } else {
        println!("\n[⏭]  exercise all — set IQOS_TEST_ALLOW_STATEFUL_WRITES=1 to enable");
    }

    println!("\n=== {passed} passed, {failed} failed ===");
    if failed > 0 {
        std::process::exit(1);
    }
}

async fn run_suite(
    name: &str,
    fut: impl std::future::Future<Output = TestResult>,
    passed: &mut usize,
    failed: &mut usize,
) {
    match fut.await {
        Ok(()) => *passed += 1,
        Err(e) => {
            println!("[❌] suite '{name}': {e}");
            *failed += 1;
        }
    }
}

// ── suites ────────────────────────────────────────────────────────────────────

async fn suite_snapshot(config: &HardwareConfig) -> TestResult {
    println!("\n=== snapshot ===");
    let session = run!("connect", connect_session(config).await);

    let model = session.model();
    let info = session.device_info();

    if model == DeviceModel::Unknown {
        println!("[❌] model: Unknown — device not recognized");
        return Err("device model should be classified".into());
    }
    println!("[✅] model: {model:?}");
    println!("     device info:");
    print_device_info(info);

    let serial = strip_nul(info.serial_number.as_deref().unwrap_or(""));
    if serial.trim().is_empty() {
        println!("[❌] serial number: missing or empty");
        return Err("serial number was not populated".into());
    }

    let battery_level = run!("read battery level", session.read_battery_level().await);
    if battery_level > 100 {
        println!("[❌] battery level: {battery_level}% (out of range)");
        return Err("battery level exceeded 100%".into());
    }
    println!("[✅] battery level: {battery_level}%");

    Ok(())
}

async fn suite_read_features(config: &HardwareConfig) -> TestResult {
    println!("\n=== read features ===");
    let session = run!("connect", connect_session(config).await);
    let model = session.model();
    let iqos = Iqos::new(session);

    let stick_fw =
        run!("read stick firmware", iqos.read_firmware_version(FirmwareKind::Stick).await);
    println!("[✅] stick firmware: {stick_fw}");

    if !model.is_one_form_factor() {
        let holder_fw =
            run!("read holder firmware", iqos.read_firmware_version(FirmwareKind::Holder).await);
        println!("[✅] holder firmware: {holder_fw}");
    }

    let voltage = run!("read battery voltage", iqos.read_battery_voltage().await);
    check_voltage(voltage)?;

    if model.supports(DeviceCapability::Vibration) {
        let vib = run!("read vibration settings", iqos.read_vibration_settings(model).await);
        println!("[✅] vibration settings: {vib:?}");
    }

    if model.supports(DeviceCapability::Brightness) {
        let brightness = run!("read brightness", iqos.read_brightness().await);
        println!("[✅] brightness: {brightness}");
    }

    if model.supports(DeviceCapability::FlexPuff) {
        let fp = run!("read FlexPuff", iqos.read_flexpuff().await);
        println!("[✅] FlexPuff: enabled={}", fp.is_enabled());
    }

    if model.supports(DeviceCapability::FlexBattery) {
        let fb = run!("read FlexBattery", iqos.read_flexbattery(model).await);
        println!("[✅] FlexBattery: mode={:?}  pause={:?}", fb.mode(), fb.pause_mode());
    }

    let diag = run!("read diagnosis", iqos.read_diagnosis().await);
    println!("[✅] diagnosis: {diag:?}");

    Ok(())
}

async fn suite_exercise_all(config: &HardwareConfig) -> TestResult {
    println!("\n=== exercise all ===");
    let session = run!("connect", connect_session(config).await);
    let model = session.model();
    let iqos = Iqos::new(session);

    // ── Step 1: snapshot ───────────────────────────────────────────────────
    println!("[Step 1] snapshot");
    if iqos.transport().model() == DeviceModel::Unknown {
        println!("[❌] [Step 1] model: Unknown — device not recognized");
        return Err("device model should be classified".into());
    }
    println!("  model: {:?}", iqos.transport().model());
    println!("  device info:");
    print_device_info(iqos.transport().device_info());
    let battery_level = run!("[Step 1] battery level", iqos.transport().read_battery_level().await);
    if battery_level > 100 {
        println!("[❌] [Step 1] battery level: {battery_level}% (out of range)");
        return Err("battery level exceeded 100%".into());
    }
    println!("  battery level: {battery_level}%");

    // ── Step 2: firmware and telemetry ─────────────────────────────────────
    println!("[Step 2] firmware and telemetry");
    let stick_fw =
        run!("[Step 2] stick firmware", iqos.read_firmware_version(FirmwareKind::Stick).await);
    println!("  stick firmware: {stick_fw}");
    if !model.is_one_form_factor() {
        let holder_fw = run!(
            "[Step 2] holder firmware",
            iqos.read_firmware_version(FirmwareKind::Holder).await
        );
        println!("  holder firmware: {holder_fw}");
    }
    let voltage = run!("[Step 2] battery voltage", iqos.read_battery_voltage().await);
    check_voltage(voltage)?;
    let diag = run!("[Step 2] diagnosis", iqos.read_diagnosis().await);
    println!("  diagnosis: {diag:?}");

    // ── Step 3: vibration controls ─────────────────────────────────────────
    println!("[Step 3] vibration controls");
    if model.supports(DeviceCapability::Vibration) {
        exercise_vibration_settings(&iqos, model).await?;
    } else {
        println!("  vibration settings: (not supported on {model:?})");
    }
    exercise_direct_vibration(&iqos, config.vibrate_millis).await?;

    // ── Step 4: brightness ─────────────────────────────────────────────────
    println!("[Step 4] brightness");
    if model.supports(DeviceCapability::Brightness) {
        exercise_brightness(&iqos).await?;
    } else {
        println!("  (not supported on {model:?})");
    }

    // ── Step 5: FlexPuff ───────────────────────────────────────────────────
    println!("[Step 5] FlexPuff");
    if model.supports(DeviceCapability::FlexPuff) {
        exercise_flexpuff(&iqos).await?;
    } else {
        println!("  (not supported on {model:?})");
    }

    // ── Step 6: FlexBattery ────────────────────────────────────────────────
    println!("[Step 6] FlexBattery");
    if model.supports(DeviceCapability::FlexBattery) {
        exercise_flexbattery(&iqos, model).await?;
    } else {
        println!("  (not supported on {model:?})");
    }

    // ── Step 7: write-only holder features ─────────────────────────────────
    println!("[Step 7] write-only holder features");
    if model.supports(DeviceCapability::SmartGesture) {
        exercise_smartgesture(&iqos, model).await?;
    } else {
        println!("  SmartGesture: (not supported on {model:?})");
    }
    if model.supports(DeviceCapability::AutoStart) {
        exercise_autostart(&iqos, model).await?;
    } else {
        println!("  AutoStart: (not supported on {model:?})");
    }

    // ── Step 8: lock/unlock ────────────────────────────────────────────────
    println!("[Step 8] lock/unlock");
    exercise_lock_unlock(&iqos, model).await?;

    Ok(())
}

// ── connection ────────────────────────────────────────────────────────────────

async fn connect_session(config: &HardwareConfig) -> TestResult<IqosBle> {
    let (peripheral, selected_name) = run!("scan for device", select_peripheral(config).await);
    println!("[✅] selected: {selected_name}");
    let session = run!("connect and discover", IqosBle::connect_and_discover(peripheral).await);
    Ok(session)
}

// ── exercise functions ────────────────────────────────────────────────────────

async fn exercise_brightness(iqos: &Iqos<IqosBle>) -> TestResult {
    let original = run!("brightness: read initial", iqos.read_brightness().await);
    let alternate = toggle_brightness(original);

    run!("brightness: set to alternate", iqos.set_brightness(alternate).await);
    let toggled_result = iqos.read_brightness().await;
    let restore_result = iqos.set_brightness(original).await;

    let toggled = run!("brightness: read after set", toggled_result);
    if toggled != alternate {
        println!("[❌] brightness: expected {alternate} after set, got {toggled}");
        restore_result?;
        return Err(
            format!("brightness did not update: expected {alternate}, got {toggled}").into()
        );
    }

    run!("brightness: restore original", restore_result);
    let restored = run!("brightness: read after restore", iqos.read_brightness().await);
    if restored != original {
        println!("[❌] brightness: expected {original} after restore, got {restored}");
        return Err(
            format!("brightness did not restore: expected {original}, got {restored}").into()
        );
    }

    println!("  [✅] brightness: {original} → {alternate} → {original}");
    Ok(())
}

async fn exercise_flexpuff(iqos: &Iqos<IqosBle>) -> TestResult {
    let original = run!("FlexPuff: read initial", iqos.read_flexpuff().await);
    let alternate = FlexPuffSetting::new(!original.is_enabled());

    run!("FlexPuff: set to alternate", iqos.set_flexpuff(alternate).await);
    let toggled_result = iqos.read_flexpuff().await;
    let restore_result = iqos.set_flexpuff(original).await;

    let toggled = run!("FlexPuff: read after set", toggled_result);
    if toggled != alternate {
        println!(
            "[❌] FlexPuff: expected enabled={} after set, got enabled={}",
            alternate.is_enabled(),
            toggled.is_enabled()
        );
        restore_result?;
        return Err("FlexPuff did not update".into());
    }

    run!("FlexPuff: restore original", restore_result);
    let restored = run!("FlexPuff: read after restore", iqos.read_flexpuff().await);
    if restored != original {
        println!(
            "[❌] FlexPuff: expected enabled={} after restore, got enabled={}",
            original.is_enabled(),
            restored.is_enabled()
        );
        return Err("FlexPuff did not restore".into());
    }

    println!(
        "  [✅] FlexPuff: {} → {} → {}",
        original.is_enabled(),
        alternate.is_enabled(),
        original.is_enabled()
    );
    Ok(())
}

async fn exercise_flexbattery(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let original = run!("FlexBattery: read initial", iqos.read_flexbattery(model).await);
    let alternate = toggle_flexbattery(original);

    run!("FlexBattery: set to alternate", iqos.set_flexbattery(model, alternate).await);
    let toggled_result = iqos.read_flexbattery(model).await;
    let restore_result = iqos.set_flexbattery(model, original).await;

    let toggled = run!("FlexBattery: read after set", toggled_result);
    if toggled != alternate {
        println!("[❌] FlexBattery: expected {alternate:?} after set, got {toggled:?}");
        restore_result?;
        return Err("FlexBattery did not update".into());
    }

    run!("FlexBattery: restore original", restore_result);
    let restored = run!("FlexBattery: read after restore", iqos.read_flexbattery(model).await);
    if restored != original {
        println!("[❌] FlexBattery: expected {original:?} after restore, got {restored:?}");
        return Err("FlexBattery did not restore".into());
    }

    println!("  [✅] FlexBattery: {original:?} → {alternate:?} → {original:?}");
    Ok(())
}

async fn exercise_vibration_settings(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let original = run!("vibration: read initial", iqos.read_vibration_settings(model).await);
    let alternate = toggle_vibration_settings(model, original);

    run!("vibration: set to alternate", iqos.update_vibration_settings(model, alternate).await);
    let toggled_result = iqos.read_vibration_settings(model).await;
    let restore_result = iqos.update_vibration_settings(model, original).await;

    let toggled = run!("vibration: read after set", toggled_result);
    if toggled != alternate {
        println!("[❌] vibration: expected {alternate:?} after set, got {toggled:?}");
        restore_result?;
        return Err("vibration settings did not update".into());
    }

    run!("vibration: restore original", restore_result);
    let restored = run!("vibration: read after restore", iqos.read_vibration_settings(model).await);
    if restored != original {
        println!("[❌] vibration: expected {original:?} after restore, got {restored:?}");
        return Err("vibration settings did not restore".into());
    }

    println!("  [✅] vibration settings: verified update and restore");
    Ok(())
}

async fn exercise_direct_vibration(iqos: &Iqos<IqosBle>, vibrate_millis: u64) -> TestResult {
    run!("vibrate_start", iqos.vibrate_start().await);
    sleep(Duration::from_millis(vibrate_millis)).await;
    run!("vibrate_stop", iqos.vibrate_stop().await);
    println!("  [✅] direct vibration: {vibrate_millis}ms");

    run!("find_my_iqos_start", iqos.find_my_iqos_start().await);
    sleep(Duration::from_millis(vibrate_millis)).await;
    run!("find_my_iqos_stop", iqos.find_my_iqos_stop().await);
    println!("  [✅] FindMyIQOS: {vibrate_millis}ms");

    Ok(())
}

async fn exercise_smartgesture(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    // No read API — command acceptance is the only verification possible.
    run!("SmartGesture: disable", iqos.set_smartgesture(model, false).await);
    run!("SmartGesture: enable", iqos.set_smartgesture(model, true).await);
    run!("SmartGesture: disable (restore)", iqos.set_smartgesture(model, false).await);
    println!("  [✅] SmartGesture: all writes accepted (left disabled)");
    Ok(())
}

async fn exercise_autostart(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    // No read API — command acceptance is the only verification possible.
    run!("AutoStart: disable", iqos.set_autostart(model, false).await);
    run!("AutoStart: enable", iqos.set_autostart(model, true).await);
    run!("AutoStart: disable (restore)", iqos.set_autostart(model, false).await);
    println!("  [✅] AutoStart: all writes accepted (left disabled)");
    Ok(())
}

async fn exercise_lock_unlock(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    // No read API — command acceptance is the only verification possible.
    run!("Lock/Unlock: unlock", iqos.unlock(model).await);
    run!("Lock/Unlock: lock", iqos.lock(model).await);
    run!("Lock/Unlock: unlock (restore)", iqos.unlock(model).await);
    println!("  [✅] Lock/Unlock: all writes accepted (left unlocked)");
    Ok(())
}

// ── display helpers ───────────────────────────────────────────────────────────

fn strip_nul(s: &str) -> &str {
    s.trim_end_matches('\0')
}

fn print_device_info(info: &DeviceInfo) {
    println!("    serial:  {}", strip_nul(info.serial_number.as_deref().unwrap_or("—")));
    println!("    model:   {}", strip_nul(info.model_number.as_deref().unwrap_or("—")));
    println!("    sw rev:  {}", strip_nul(info.software_revision.as_deref().unwrap_or("—")));
    println!("    mfr:     {}", strip_nul(info.manufacturer_name.as_deref().unwrap_or("—")));
}

fn check_voltage(voltage: f32) -> TestResult {
    if !(2.5..=4.4).contains(&voltage) {
        println!("[❌] battery voltage: {voltage:.3}V outside expected range 2.5–4.4V");
        return Err(format!("battery voltage {voltage:.3}V out of range").into());
    }
    println!("[✅] battery voltage: {voltage:.3}V");
    Ok(())
}

// ── misc helpers ──────────────────────────────────────────────────────────────

fn env_flag(name: &str) -> TestResult<bool> {
    match env::var(name) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" | "" => Ok(false),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid {name} value: {other}"),
            )
            .into()),
        },
        Err(_) => Ok(false),
    }
}

fn toggle_brightness(level: BrightnessLevel) -> BrightnessLevel {
    match level {
        BrightnessLevel::High => BrightnessLevel::Low,
        BrightnessLevel::Low => BrightnessLevel::High,
    }
}

fn toggle_flexbattery(settings: FlexBatterySettings) -> FlexBatterySettings {
    let alternate_mode = match settings.mode() {
        FlexBatteryMode::Performance => FlexBatteryMode::Eco,
        FlexBatteryMode::Eco => FlexBatteryMode::Performance,
    };
    FlexBatterySettings::new(alternate_mode, settings.pause_mode().map(|v| !v))
}

fn toggle_vibration_settings(model: DeviceModel, settings: VibrationSettings) -> VibrationSettings {
    let heating_start = !settings.when_heating_start();
    let starting_to_use = settings.when_starting_to_use();
    let puff_end = settings.when_puff_end();
    let manually_terminated = settings.when_manually_terminated();

    if model.supports_charge_start_vibration() {
        VibrationSettings::with_charge_start(
            heating_start,
            starting_to_use,
            puff_end,
            manually_terminated,
            !settings.when_charging_start().unwrap_or(false),
        )
    } else {
        VibrationSettings::new(heating_start, starting_to_use, puff_end, manually_terminated)
    }
}

async fn select_peripheral(config: &HardwareConfig) -> TestResult<(Peripheral, String)> {
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no Bluetooth adapter found"))?;

    match &config.name_filter {
        Some(filter) => println!(
            "Scanning for IQOS devices matching {:?} for {} seconds...",
            filter, config.scan_seconds
        ),
        None => println!("Scanning for any IQOS device for {} seconds...", config.scan_seconds),
    }
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(config.scan_seconds)).await;
    adapter.stop_scan().await?;

    let normalized_filter = config.name_filter.as_deref().map(str::to_ascii_lowercase);
    let mut candidates = Vec::new();

    for peripheral in adapter.peripherals().await? {
        let Some(properties) = peripheral.properties().await? else {
            continue;
        };
        let Some(name) = properties.local_name else {
            continue;
        };
        if !name.to_ascii_uppercase().contains("IQOS") {
            continue;
        }
        if let Some(filter) = &normalized_filter
            && !name.to_ascii_lowercase().contains(filter.as_str())
        {
            continue;
        }
        candidates.push((name, peripheral));
    }

    candidates.sort_by(|l, r| l.0.cmp(&r.0));
    candidates.into_iter().next().map(|(name, peripheral)| (peripheral, name)).ok_or_else(|| {
        let detail = match &config.name_filter {
            Some(f) => format!("no IQOS peripheral matched IQOS_TEST_NAME_SUBSTRING={f:?}"),
            None => "no IQOS peripheral found nearby".to_string(),
        };
        io::Error::new(io::ErrorKind::NotFound, detail).into()
    })
}
