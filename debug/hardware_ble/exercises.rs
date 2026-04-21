use iqos::{
    BrightnessLevel, DeviceModel, FlexBatteryMode, FlexBatterySettings, FlexPuffSetting, Iqos,
    IqosBle, VibrationSettings,
};
use tokio::time::{Duration, sleep, timeout};

use crate::TestResult;

const STATUS_READ_TIMEOUT: Duration = Duration::from_millis(120);
const AUTOSTART_STATUS_SUBTYPE: u8 = 0x01;
const SMARTGESTURE_STATUS_SUBTYPE: u8 = 0x04;

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

    println!("    current brightness status: {original}");
    println!("    request: set brightness -> {toggled}");
    run!(format!("set brightness → {toggled}"), iqos.set_brightness(toggled).await);

    let read_back = run!("verify brightness", iqos.read_brightness().await);
    if read_back != toggled {
        println!("    verify: brightness changed to {read_back} [FAIL]");
        println!("    request: set brightness -> {original}");
        let restore_write = iqos.set_brightness(original).await;
        let restore_read = iqos.read_brightness().await;
        return Err(match (restore_write, restore_read) {
            (Ok(()), Ok(restored)) if restored == original => format!(
                "brightness mismatch: expected {toggled}, got {read_back}; restore verified to {restored}"
            )
            .into(),
            (Ok(()), Ok(restored)) => format!(
                "brightness mismatch: expected {toggled}, got {read_back}; restore verification failed: expected {original}, got {restored}"
            )
            .into(),
            (write_result, read_result) => format!(
                "brightness mismatch: expected {toggled}, got {read_back}; restore write={:?}, restore read={:?}",
                write_result.err(),
                read_result.err()
            )
            .into(),
        });
    }
    println!("    verify: brightness changed to {read_back} [PASS]");

    println!("    request: set brightness -> {original}");
    run!(format!("restore brightness → {original}"), iqos.set_brightness(original).await);

    let restored = run!("verify brightness restored", iqos.read_brightness().await);
    if restored != original {
        return Err(format!("restore failed: expected {original}, got {restored}").into());
    }
    println!("    verify: brightness changed to {restored} [PASS]");

    Ok(())
}

pub(crate) async fn vibration_settings(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let original = run!("read vibration settings", iqos.read_vibration_settings(model).await);

    let toggled = if let Some(charge_start) = original.when_charging_start() {
        VibrationSettings::with_charge_start(
            !original.when_heating_start(),
            !original.when_starting_to_use(),
            !original.when_puff_end(),
            !original.when_manually_terminated(),
            !charge_start,
        )
    } else {
        VibrationSettings::new(
            !original.when_heating_start(),
            !original.when_starting_to_use(),
            !original.when_puff_end(),
            !original.when_manually_terminated(),
        )
    };

    println!("    current vibration status: {}", format_vibration_settings(original));
    println!("    request: set vibration status -> {}", format_vibration_settings(toggled));
    run!("write vibration settings", iqos.update_vibration_settings(model, toggled).await);

    let read_back = run!("verify vibration settings", iqos.read_vibration_settings(model).await);
    if read_back != toggled {
        println!(
            "    verify: vibration status changed to {} [FAIL]",
            format_vibration_settings(read_back)
        );
        println!("    request: set vibration status -> {}", format_vibration_settings(original));
        let restore_write = iqos.update_vibration_settings(model, original).await;
        let restore_read = iqos.read_vibration_settings(model).await;
        let restore_summary = match (restore_write, restore_read) {
            (Ok(()), Ok(restored)) if restored == original => {
                format!("restore verified to {}", format_vibration_settings(restored))
            }
            (Ok(()), Ok(restored)) => format!(
                "restore verification failed: expected {}, got {}",
                format_vibration_settings(original),
                format_vibration_settings(restored)
            ),
            (write_result, read_result) => format!(
                "restore write={:?}, restore read={:?}",
                write_result.err(),
                read_result.err()
            ),
        };
        return Err(format!(
            "vibration mismatch: expected {}, got {}; {}",
            format_vibration_settings(toggled),
            format_vibration_settings(read_back),
            restore_summary
        )
        .into());
    }
    println!(
        "    verify: vibration status changed to {} [PASS]",
        format_vibration_settings(read_back)
    );

    println!("    request: set vibration status -> {}", format_vibration_settings(original));
    run!("restore vibration settings", iqos.update_vibration_settings(model, original).await);

    let restored = run!("verify vibration restored", iqos.read_vibration_settings(model).await);
    if restored != original {
        return Err(format!(
            "restore failed: expected {}, got {}",
            format_vibration_settings(original),
            format_vibration_settings(restored),
        )
        .into());
    }
    println!(
        "    verify: vibration status changed to {} [PASS]",
        format_vibration_settings(restored)
    );

    Ok(())
}

pub(crate) async fn direct_vibration(iqos: &Iqos<IqosBle>, millis: u64) -> TestResult {
    println!("    request: vibrate start");
    run!("vibrate start", iqos.vibrate_start().await);
    sleep(Duration::from_millis(millis)).await;
    println!("    request: vibrate stop");
    run!("vibrate stop", iqos.vibrate_stop().await);
    Ok(())
}

pub(crate) async fn lock_unlock(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    println!("    request: lock");
    run!("lock", iqos.lock(model).await);
    println!("    request: unlock");
    run!("unlock", iqos.unlock(model).await);
    Ok(())
}

pub(crate) async fn flexpuff(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let original = run!("read flexpuff", iqos.read_flexpuff(model).await);
    let toggled = FlexPuffSetting::new(!original.is_enabled());
    let label = bool_label(toggled.is_enabled());

    println!("    current flexpuff status: {}", bool_label(original.is_enabled()));
    println!("    request: set flexpuff -> {label}");
    run!(format!("set flexpuff → {label}"), iqos.set_flexpuff(model, toggled).await);

    let read_back = run!("verify flexpuff", iqos.read_flexpuff(model).await);
    if read_back != toggled {
        println!("    verify: flexpuff changed to {} [FAIL]", bool_label(read_back.is_enabled()));
        println!("    request: set flexpuff -> {}", bool_label(original.is_enabled()));
        let restore_write = iqos.set_flexpuff(model, original).await;
        let restore_read = iqos.read_flexpuff(model).await;
        return Err(match (restore_write, restore_read) {
            (Ok(()), Ok(restored)) if restored == original => format!(
                "flexpuff mismatch: expected {:?}, got {:?}; restore verified to {:?}",
                toggled, read_back, restored
            )
            .into(),
            (Ok(()), Ok(restored)) => format!(
                "flexpuff mismatch: expected {:?}, got {:?}; restore verification failed: expected {:?}, got {:?}",
                toggled, read_back, original, restored
            )
            .into(),
            (write_result, read_result) => format!(
                "flexpuff mismatch: expected {:?}, got {:?}; restore write={:?}, restore read={:?}",
                toggled,
                read_back,
                write_result.err(),
                read_result.err()
            )
            .into(),
        });
    }
    println!("    verify: flexpuff changed to {} [PASS]", bool_label(read_back.is_enabled()));

    let orig_label = bool_label(original.is_enabled());
    println!("    request: set flexpuff -> {orig_label}");
    run!(format!("restore flexpuff → {orig_label}"), iqos.set_flexpuff(model, original).await);

    let restored = run!("verify flexpuff restored", iqos.read_flexpuff(model).await);
    if restored != original {
        return Err(format!(
            "flexpuff restore failed: expected {:?}, got {:?}",
            original, restored
        )
        .into());
    }
    println!("    verify: flexpuff changed to {} [PASS]", bool_label(restored.is_enabled()));

    Ok(())
}

pub(crate) async fn flexbattery(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let original = run!("read flexbattery", iqos.read_flexbattery(model).await);
    let toggled_mode = match original.mode() {
        FlexBatteryMode::Performance => FlexBatteryMode::Eco,
        FlexBatteryMode::Eco => FlexBatteryMode::Performance,
    };
    let toggled = FlexBatterySettings::new(toggled_mode, original.pause_mode().map(|value| !value));

    println!("    current flexbattery status: {}", format_flexbattery_settings(original));
    println!("    request: set flexbattery -> {}", format_flexbattery_settings(toggled));
    run!(format!("set flexbattery → {toggled_mode:?}"), iqos.set_flexbattery(model, toggled).await);

    let read_back = run!("verify flexbattery", iqos.read_flexbattery(model).await);
    if read_back != toggled {
        println!(
            "    verify: flexbattery changed to {} [FAIL]",
            format_flexbattery_settings(read_back)
        );
        println!("    request: set flexbattery -> {}", format_flexbattery_settings(original));
        let restore_write = iqos.set_flexbattery(model, original).await;
        let restore_read = iqos.read_flexbattery(model).await;
        let restore_summary = match (restore_write, restore_read) {
            (Ok(()), Ok(restored)) if restored == original => {
                format!("restore verified to {}", format_flexbattery_settings(restored))
            }
            (Ok(()), Ok(restored)) => format!(
                "restore verification failed: expected {}, got {}",
                format_flexbattery_settings(original),
                format_flexbattery_settings(restored)
            ),
            (write_result, read_result) => format!(
                "restore write={:?}, restore read={:?}",
                write_result.err(),
                read_result.err()
            ),
        };
        return Err(format!(
            "flexbattery mismatch: expected {}, got {}; {}",
            format_flexbattery_settings(toggled),
            format_flexbattery_settings(read_back),
            restore_summary
        )
        .into());
    }
    println!(
        "    verify: flexbattery changed to {} [PASS]",
        format_flexbattery_settings(read_back)
    );

    println!("    request: set flexbattery -> {}", format_flexbattery_settings(original));
    run!(
        format!("restore flexbattery → {:?}", original.mode()),
        iqos.set_flexbattery(model, original).await
    );

    let restored = run!("verify flexbattery restored", iqos.read_flexbattery(model).await);
    if restored != original {
        return Err(format!(
            "restore failed: expected {}, got {}",
            format_flexbattery_settings(original),
            format_flexbattery_settings(restored),
        )
        .into());
    }
    println!("    verify: flexbattery changed to {} [PASS]", format_flexbattery_settings(restored));

    Ok(())
}

pub(crate) async fn smartgesture(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let (status_command, original) =
        discover_toggle_status_command(iqos, "smartgesture", SMARTGESTURE_STATUS_SUBTYPE).await?;
    let toggled = !original;

    println!("    current smartgesture status: {}", bool_label(original));
    println!("    request: set smartgesture -> {}", bool_label(toggled));
    run!(
        format!("set smartgesture → {}", bool_label(toggled)),
        iqos.set_smartgesture(model, toggled).await
    );

    let read_back = run!(
        "verify smartgesture",
        read_toggle_status(iqos, "smartgesture", SMARTGESTURE_STATUS_SUBTYPE, &status_command)
            .await
    );
    if read_back != toggled {
        println!("    verify: smartgesture changed to {} [FAIL]", bool_label(read_back));
        println!("    request: set smartgesture -> {}", bool_label(original));
        let restore_write = iqos.set_smartgesture(model, original).await;
        let restore_read =
            read_toggle_status(iqos, "smartgesture", SMARTGESTURE_STATUS_SUBTYPE, &status_command)
                .await;
        let restore_summary = match (restore_write, restore_read) {
            (Ok(()), Ok(restored)) if restored == original => {
                format!("restore verified to {}", bool_label(restored))
            }
            (Ok(()), Ok(restored)) => format!(
                "restore verification failed: expected {}, got {}",
                bool_label(original),
                bool_label(restored)
            ),
            (write_result, read_result) => format!(
                "restore write={:?}, restore read={:?}",
                write_result.err(),
                read_result.err()
            ),
        };
        return Err(format!(
            "smartgesture mismatch: expected {}, got {}; {}",
            bool_label(toggled),
            bool_label(read_back),
            restore_summary
        )
        .into());
    }
    println!("    verify: smartgesture changed to {} [PASS]", bool_label(read_back));

    println!("    request: set smartgesture -> {}", bool_label(original));
    run!(
        format!("restore smartgesture → {}", bool_label(original)),
        iqos.set_smartgesture(model, original).await
    );

    let restored = run!(
        "verify smartgesture restored",
        read_toggle_status(iqos, "smartgesture", SMARTGESTURE_STATUS_SUBTYPE, &status_command)
            .await
    );
    if restored != original {
        return Err(format!(
            "smartgesture restore failed: expected {}, got {}",
            bool_label(original),
            bool_label(restored)
        )
        .into());
    }
    println!("    verify: smartgesture changed to {} [PASS]", bool_label(restored));

    Ok(())
}

pub(crate) async fn autostart(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult {
    let (status_command, original) =
        discover_toggle_status_command(iqos, "autostart", AUTOSTART_STATUS_SUBTYPE).await?;
    let toggled = !original;

    println!("    current autostart status: {}", bool_label(original));
    println!("    request: set autostart -> {}", bool_label(toggled));
    run!(
        format!("set autostart → {}", bool_label(toggled)),
        iqos.set_autostart(model, toggled).await
    );

    let read_back = run!(
        "verify autostart",
        read_toggle_status(iqos, "autostart", AUTOSTART_STATUS_SUBTYPE, &status_command).await
    );
    if read_back != toggled {
        println!("    verify: autostart changed to {} [FAIL]", bool_label(read_back));
        println!("    request: set autostart -> {}", bool_label(original));
        let restore_write = iqos.set_autostart(model, original).await;
        let restore_read =
            read_toggle_status(iqos, "autostart", AUTOSTART_STATUS_SUBTYPE, &status_command).await;
        let restore_summary = match (restore_write, restore_read) {
            (Ok(()), Ok(restored)) if restored == original => {
                format!("restore verified to {}", bool_label(restored))
            }
            (Ok(()), Ok(restored)) => format!(
                "restore verification failed: expected {}, got {}",
                bool_label(original),
                bool_label(restored)
            ),
            (write_result, read_result) => format!(
                "restore write={:?}, restore read={:?}",
                write_result.err(),
                read_result.err()
            ),
        };
        return Err(format!(
            "autostart mismatch: expected {}, got {}; {}",
            bool_label(toggled),
            bool_label(read_back),
            restore_summary
        )
        .into());
    }
    println!("    verify: autostart changed to {} [PASS]", bool_label(read_back));

    println!("    request: set autostart -> {}", bool_label(original));
    run!(
        format!("restore autostart → {}", bool_label(original)),
        iqos.set_autostart(model, original).await
    );

    let restored = run!(
        "verify autostart restored",
        read_toggle_status(iqos, "autostart", AUTOSTART_STATUS_SUBTYPE, &status_command).await
    );
    if restored != original {
        return Err(format!(
            "autostart restore failed: expected {}, got {}",
            bool_label(original),
            bool_label(restored)
        )
        .into());
    }
    println!("    verify: autostart changed to {} [PASS]", bool_label(restored));

    Ok(())
}

async fn discover_toggle_status_command(
    iqos: &Iqos<IqosBle>,
    label: &str,
    subtype: u8,
) -> TestResult<(Vec<u8>, bool)> {
    println!("    probing experimental {label} status read command...");

    for checksum in checksum_candidates() {
        let command = vec![0x00, 0xC9, 0x07, 0x24, subtype, 0x00, 0x00, 0x00, checksum];
        let Some(bytes) = request_optional(iqos, &command).await? else {
            continue;
        };
        let Some(status) = parse_toggle_status(&bytes, subtype) else {
            continue;
        };

        if !confirm_toggle_status_command(iqos, label, subtype, &command, status).await? {
            continue;
        }

        println!("    discovered {label} status command: {}", hex_bytes(&command));
        println!("    discovered {label} status frame:   {}", hex_bytes(&bytes));
        return Ok((command, status));
    }

    Err(format!("unable to discover {label} status read command").into())
}

async fn read_toggle_status(
    iqos: &Iqos<IqosBle>,
    label: &str,
    subtype: u8,
    command: &[u8],
) -> TestResult<bool> {
    let bytes = timeout(STATUS_READ_TIMEOUT, iqos.transport().request(command))
        .await
        .map_err(|_| format!("{label} status request timed out"))??;

    parse_toggle_status(&bytes, subtype)
        .ok_or_else(|| format!("unexpected {label} status frame: {}", hex_bytes(&bytes),).into())
}

async fn request_optional(iqos: &Iqos<IqosBle>, command: &[u8]) -> TestResult<Option<Vec<u8>>> {
    match timeout(STATUS_READ_TIMEOUT, iqos.transport().request(command)).await {
        Ok(Ok(bytes)) => Ok(Some(bytes)),
        Ok(Err(error)) => {
            println!(
                "    [⚠] ignoring experimental status-probe error for {}: {error}",
                hex_bytes(command)
            );
            Ok(None)
        }
        Err(_) => Ok(None),
    }
}

async fn confirm_toggle_status_command(
    iqos: &Iqos<IqosBle>,
    label: &str,
    subtype: u8,
    command: &[u8],
    expected: bool,
) -> TestResult<bool> {
    let Some(bytes) = request_optional(iqos, command).await? else {
        println!("    [⚠] {label} status probe did not confirm on second read");
        return Ok(false);
    };

    let Some(status) = parse_toggle_status(&bytes, subtype) else {
        println!(
            "    [⚠] {label} status probe returned an unexpected confirmation frame: {}",
            hex_bytes(&bytes)
        );
        return Ok(false);
    };

    if status != expected {
        println!(
            "    [⚠] {label} status probe was unstable across repeated reads: first={}, second={}",
            bool_label(expected),
            bool_label(status)
        );
        return Ok(false);
    }

    Ok(true)
}

fn parse_toggle_status(bytes: &[u8], subtype: u8) -> Option<bool> {
    if bytes.len() != 9
        || bytes[0] != 0x00
        || bytes[1] != 0x08
        || bytes[2] != 0x87
        || bytes[3] != 0x24
        || bytes[4] != subtype
        || bytes[6] != 0x00
        || bytes[7] != 0x00
        || bytes[8] != 0x00
    {
        return None;
    }

    match bytes[5] {
        0x00 => Some(false),
        0x01 => Some(true),
        _ => None,
    }
}

fn checksum_candidates() -> Vec<u8> {
    let mut candidates = vec![
        0x18, 0x08, 0x17, 0x16, 0x19, 0x1A, 0x1F, 0x20, 0x21, 0x22, 0x2C, 0x2F, 0xC2, 0xCF, 0xFD,
        0xFE,
    ];

    for checksum in u8::MIN..=u8::MAX {
        if !candidates.contains(&checksum) {
            candidates.push(checksum);
        }
    }

    candidates
}

fn bool_label(enabled: bool) -> &'static str {
    if enabled { "enabled" } else { "disabled" }
}

fn format_flexbattery_settings(settings: FlexBatterySettings) -> String {
    format!(
        "mode={:?}, pause={}",
        settings.mode(),
        settings.pause_mode().map(bool_label).unwrap_or("unavailable")
    )
}

fn format_vibration_settings(settings: VibrationSettings) -> String {
    let charge_start = settings
        .when_charging_start()
        .map(|value| format!(", charge_start={}", bool_label(value)))
        .unwrap_or_default();

    format!(
        "heating_start={}, starting_to_use={}, puff_end={}, manually_terminated={}{}",
        bool_label(settings.when_heating_start()),
        bool_label(settings.when_starting_to_use()),
        bool_label(settings.when_puff_end()),
        bool_label(settings.when_manually_terminated()),
        charge_start
    )
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect::<Vec<_>>().join(" ")
}
