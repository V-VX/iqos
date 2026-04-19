use std::io;

use btleplug::api::{Central as _, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use iqos::{
    DeviceCapability, DeviceModel, Iqos, IqosBle,
    protocol::{LOAD_AUTOSTART_COMMAND, autostart_command},
};
use tokio::time::{Duration, sleep};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const DEFAULT_REQUEST_INTERVAL_MILLIS: u64 = 200;
const SCAN_SECONDS: u64 = 5;

struct Config {
    name_filter: Option<String>,
    request_interval_millis: u64,
    stateful_writes: bool,
}

fn load_config() -> TestResult<Config> {
    let name_filter =
        std::env::var("IQOS_TEST_NAME_SUBSTRING").ok().filter(|value| !value.is_empty());
    let request_interval_millis = match std::env::var("IQOS_TEST_REQUEST_INTERVAL_MILLIS") {
        Ok(value) => value.parse::<u64>().map_err(|error| {
            io::Error::other(format!(
                "failed to parse IQOS_TEST_REQUEST_INTERVAL_MILLIS={value:?} as milliseconds: {error}"
            ))
        })?,
        Err(std::env::VarError::NotPresent) => DEFAULT_REQUEST_INTERVAL_MILLIS,
        Err(error) => {
            return Err(io::Error::other(format!(
                "failed to read IQOS_TEST_REQUEST_INTERVAL_MILLIS: {error}"
            ))
            .into());
        }
    };

    Ok(Config {
        name_filter,
        request_interval_millis,
        stateful_writes: std::env::var("IQOS_TEST_ALLOW_STATEFUL_WRITES")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
    })
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("\nfatal: {error}");
        std::process::exit(1);
    }
}

async fn run() -> TestResult {
    let config = load_config()?;
    if !config.stateful_writes {
        return Err(io::Error::other(
            "stateful writes are disabled; rerun with IQOS_TEST_ALLOW_STATEFUL_WRITES=1",
        )
        .into());
    }

    println!("=== IQOS Auto Start Hardware Debug ===\n");
    println!("Stateful writes: enabled");
    println!("Request interval: {} ms", config.request_interval_millis);

    let filter_hint = config
        .name_filter
        .as_deref()
        .map(|filter| format!(" (filter: \"{filter}\")"))
        .unwrap_or_default();
    println!("Connecting{filter_hint}...");

    let session = scan_and_connect(config.name_filter.as_deref()).await?;
    let model = session.model();

    println!("[ok] Connected model={model:?}\n");

    if !model.supports(DeviceCapability::AutoStart) {
        return Err(io::Error::other(format!(
            "Auto Start is not supported for connected model {model:?}"
        ))
        .into());
    }

    let iqos = Iqos::new(session);
    exercise_autostart(&iqos, model, &config).await
}

async fn exercise_autostart(
    iqos: &Iqos<IqosBle>,
    model: DeviceModel,
    config: &Config,
) -> TestResult {
    println!("--- Read Current Auto Start Status ---");
    let original = read_autostart_status(iqos, model).await?;
    let target = !original;
    println!("  original: {}", format_autostart(original));
    println!("  target:   {}\n", format_autostart(target));

    settle_interval(config.request_interval_millis, "before sending opposite-value write").await;

    println!("--- Send Opposite Auto Start Value ---");
    println!("  request: {}", format_command(autostart_command(target)));
    println!("  action:  set Auto Start {}\n", format_autostart(target));

    if let Err(error) = iqos.set_autostart(model, target).await {
        settle_interval(
            config.request_interval_millis,
            "before attempting restoration after failed write",
        )
        .await;
        let restore_outcome = restore_after_failure(iqos, model, original, config).await;
        return Err(io::Error::other(format!(
            "failed to send Auto Start update to {}: {error}. Restoration outcome: {restore_outcome}",
            format_autostart(target),
        ))
        .into());
    }

    settle_interval(config.request_interval_millis, "after update write before read-back").await;

    println!("--- Verify Auto Start Change ---");
    match read_autostart_status(iqos, model).await {
        Ok(observed) if observed == target => {
            println!("  verification: OK ({})\n", format_autostart(observed));
        }
        Ok(observed) => {
            println!(
                "  verification: FAILED (expected {}, observed {})\n",
                format_autostart(target),
                format_autostart(observed),
            );
            settle_interval(
                config.request_interval_millis,
                "before attempting restoration after failed verification",
            )
            .await;
            let restore_outcome = restore_after_failure(iqos, model, original, config).await;
            return Err(io::Error::other(format!(
                "Auto Start verification failed after update: expected {}, observed {}. Restoration outcome: {restore_outcome}",
                format_autostart(target),
                format_autostart(observed),
            ))
            .into());
        }
        Err(error) => {
            println!("  verification: FAILED ({error})\n");
            settle_interval(
                config.request_interval_millis,
                "before attempting restoration after failed verification read",
            )
            .await;
            let restore_outcome = restore_after_failure(iqos, model, original, config).await;
            return Err(io::Error::other(format!(
                "failed to read Auto Start after update: {error}. Restoration outcome: {restore_outcome}"
            ))
            .into());
        }
    }

    settle_interval(config.request_interval_millis, "before sending restore write").await;
    restore_autostart(iqos, model, original, config).await?;

    println!("=== Auto Start reversible validation passed ===");
    Ok(())
}

async fn restore_autostart(
    iqos: &Iqos<IqosBle>,
    model: DeviceModel,
    original: bool,
    config: &Config,
) -> TestResult {
    println!("--- Restore Original Auto Start Value ---");
    println!("  request: {}", format_command(autostart_command(original)));
    println!("  action:  restore Auto Start {}\n", format_autostart(original));

    iqos.set_autostart(model, original).await?;

    settle_interval(config.request_interval_millis, "after restore write before read-back").await;

    println!("--- Verify Auto Start Restoration ---");
    let restored = read_autostart_status(iqos, model).await?;
    if restored != original {
        return Err(io::Error::other(format!(
            "restore verification failed: expected {}, observed {}",
            format_autostart(original),
            format_autostart(restored),
        ))
        .into());
    }

    println!("  restore verification: OK ({})\n", format_autostart(restored));
    Ok(())
}

async fn restore_after_failure(
    iqos: &Iqos<IqosBle>,
    model: DeviceModel,
    original: bool,
    config: &Config,
) -> String {
    match restore_autostart(iqos, model, original, config).await {
        Ok(()) => format!("restored to {}", format_autostart(original)),
        Err(error) => format!("FAILED ({error})"),
    }
}

async fn read_autostart_status(iqos: &Iqos<IqosBle>, model: DeviceModel) -> TestResult<bool> {
    println!("  request: {}", format_command(&LOAD_AUTOSTART_COMMAND));
    let enabled = iqos.read_autostart(model).await?;
    println!("  status:  {}\n", format_autostart(enabled));
    Ok(enabled)
}

async fn settle_interval(request_interval_millis: u64, context: &str) {
    println!("  settle: waiting {request_interval_millis} ms ({context})\n");
    sleep(Duration::from_millis(request_interval_millis)).await;
}

async fn scan_and_connect(name_filter: Option<&str>) -> TestResult<IqosBle> {
    let adapter = first_adapter().await?;
    println!("  scanning for {SCAN_SECONDS} s...");
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(SCAN_SECONDS)).await;
    adapter.stop_scan().await?;
    let peripheral = select_peripheral(&adapter, name_filter).await?;
    Ok(IqosBle::connect_and_discover(peripheral).await?)
}

async fn first_adapter() -> TestResult<Adapter> {
    Manager::new()
        .await?
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| "no Bluetooth adapter found".into())
}

async fn select_peripheral(adapter: &Adapter, name_filter: Option<&str>) -> TestResult<Peripheral> {
    let normalized_filter = name_filter.map(str::to_ascii_lowercase);
    let mut candidates: Vec<(String, Peripheral)> = Vec::new();

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
        if normalized_filter
            .as_ref()
            .is_some_and(|filter| !name.to_ascii_lowercase().contains(filter.as_str()))
        {
            continue;
        }
        candidates.push((name, peripheral));
    }

    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    candidates.into_iter().next().map(|(_, peripheral)| peripheral).ok_or_else(|| {
        let hint = name_filter.map(|filter| format!(" matching \"{filter}\"")).unwrap_or_default();
        format!("no IQOS device found{hint}").into()
    })
}

fn format_autostart(enabled: bool) -> &'static str {
    if enabled { "enabled" } else { "disabled" }
}

fn format_command(command: &[u8]) -> String {
    command.iter().map(|byte| format!("{byte:02X}")).collect::<Vec<_>>().join(" ")
}
