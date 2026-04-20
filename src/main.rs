#[cfg_attr(not(any(feature = "btleplug-support", test)), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArgs {
    name_filter: Option<String>,
    command: Command,
}

#[cfg_attr(not(any(feature = "btleplug-support", test)), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Inspect,
    Probe(ProbeCommand),
}

#[cfg_attr(not(any(feature = "btleplug-support", test)), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeCommand {
    Brightness,
    FirmwareStick,
    FirmwareHolder,
    Battery,
}

fn usage() -> &'static str {
    "Usage: cargo run --features btleplug-support -- <inspect|probe> [subcommand] [--name <substring>]

Commands:
  inspect
  probe brightness
  probe firmware-stick
  probe firmware-holder
  probe battery"
}

#[cfg_attr(not(any(feature = "btleplug-support", test)), allow(dead_code))]
fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliArgs, String> {
    let mut args = args.into_iter();
    let _binary = args.next();

    let mut name_filter = None;
    let mut positional = Vec::new();

    while let Some(arg) = args.next() {
        if arg == "--name" {
            let value = args.next().ok_or_else(|| "missing value for --name".to_string())?;
            name_filter = Some(value);
        } else if let Some(value) = arg.strip_prefix("--name=") {
            if value.is_empty() {
                return Err("missing value for --name".to_string());
            }
            name_filter = Some(value.to_string());
        } else if arg.starts_with("--") {
            return Err(format!("unknown option: {arg}"));
        } else {
            positional.push(arg);
        }
    }

    let command = match positional.as_slice() {
        [command] if command == "inspect" => Command::Inspect,
        [command, probe] if command == "probe" => Command::Probe(
            parse_probe_command(probe)
                .ok_or_else(|| format!("unknown probe subcommand: {probe}"))?,
        ),
        [] => return Err("missing command".to_string()),
        _ => return Err("invalid command shape".to_string()),
    };

    Ok(CliArgs { name_filter, command })
}

#[cfg_attr(not(any(feature = "btleplug-support", test)), allow(dead_code))]
fn parse_probe_command(value: &str) -> Option<ProbeCommand> {
    match value {
        "brightness" => Some(ProbeCommand::Brightness),
        "firmware-stick" => Some(ProbeCommand::FirmwareStick),
        "firmware-holder" => Some(ProbeCommand::FirmwareHolder),
        "battery" => Some(ProbeCommand::Battery),
        _ => None,
    }
}

#[cfg(feature = "btleplug-support")]
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error}");
        eprintln!();
        eprintln!("{}", usage());
        std::process::exit(1);
    }
}

#[cfg(not(feature = "btleplug-support"))]
fn main() {
    eprintln!("the debug CLI requires the `btleplug-support` feature");
    eprintln!();
    eprintln!("{}", usage());
    std::process::exit(1);
}

#[cfg(feature = "btleplug-support")]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    use std::io;

    let args = parse_args(std::env::args()).map_err(io::Error::other)?;
    match args.command {
        Command::Inspect => inspect_device(args.name_filter.as_deref()).await,
        Command::Probe(command) => probe_device(command, args.name_filter.as_deref()).await,
    }
}

#[cfg(feature = "btleplug-support")]
async fn inspect_device(name_filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    use iqos::{Iqos, IqosBle};

    let (peripheral, selected_name) = select_peripheral(name_filter).await?;
    println!("Selected device: {selected_name}");

    ensure_connected_and_discovered(&peripheral).await?;
    print_service_summary(&peripheral);

    let session = IqosBle::connect_and_discover(peripheral).await?;
    let model = session.model();
    println!("Model: {model:?}");
    println!("Device information:");
    print_device_info(session.device_info());

    match session.read_battery_level().await {
        Ok(level) => println!("Battery level (GATT): {level}%"),
        Err(error) => println!("Battery level (GATT): read failed ({error})"),
    }

    let iqos = Iqos::new(session);
    match iqos.read_device_status(model).await {
        Ok(status) => print_device_status(&status),
        Err(error) => println!("Device status: read failed ({error})"),
    }

    Ok(())
}

#[cfg(feature = "btleplug-support")]
async fn probe_device(
    command: ProbeCommand,
    name_filter: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use iqos::{FirmwareKind, Iqos, IqosBle};

    let (peripheral, selected_name) = select_peripheral(name_filter).await?;
    println!("Selected device: {selected_name}");

    let session = IqosBle::connect_and_discover(peripheral).await?;
    let iqos = Iqos::new(session.clone());

    match command {
        ProbeCommand::Brightness => {
            let brightness = iqos.read_brightness().await?;
            println!("Brightness: {brightness}");
        }
        ProbeCommand::FirmwareStick => {
            let firmware = iqos.read_firmware_version(FirmwareKind::Stick).await?;
            println!("Stick firmware: {firmware}");
        }
        ProbeCommand::FirmwareHolder => {
            let firmware = iqos.read_firmware_version(FirmwareKind::Holder).await?;
            println!("Holder firmware: {firmware}");
        }
        ProbeCommand::Battery => {
            let level = session.read_battery_level().await?;
            println!("Battery level: {level}%");
        }
    }

    Ok(())
}

#[cfg(feature = "btleplug-support")]
async fn select_peripheral(
    name_filter: Option<&str>,
) -> Result<(btleplug::platform::Peripheral, String), Box<dyn std::error::Error>> {
    use std::io;

    use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
    use btleplug::platform::Manager;
    use tokio::time::{Duration, sleep};

    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no Bluetooth adapter found"))?;

    println!("Scanning for IQOS devices...");
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(3)).await;
    adapter.stop_scan().await?;

    let normalized_filter = name_filter.map(|value| value.to_ascii_lowercase());
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
            && !name.to_ascii_lowercase().contains(filter)
        {
            continue;
        }

        candidates.push((name, peripheral));
    }

    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    candidates.into_iter().next().map(|(name, peripheral)| (peripheral, name)).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "no matching IQOS device found").into()
    })
}

#[cfg(feature = "btleplug-support")]
async fn ensure_connected_and_discovered(
    peripheral: &btleplug::platform::Peripheral,
) -> Result<(), Box<dyn std::error::Error>> {
    use btleplug::api::Peripheral as _;

    if !peripheral.is_connected().await? {
        peripheral.connect().await?;
    }
    peripheral.discover_services().await?;
    Ok(())
}

#[cfg(feature = "btleplug-support")]
fn print_service_summary(peripheral: &btleplug::platform::Peripheral) {
    use btleplug::api::Peripheral as _;

    let mut services: Vec<_> = peripheral.services().into_iter().collect();
    services.sort_by(|left, right| left.uuid.as_hyphenated().cmp(right.uuid.as_hyphenated()));

    println!("Services:");
    if services.is_empty() {
        println!("  (none)");
        return;
    }

    for service in services {
        println!("  {} [{}]", service.uuid, if service.primary { "primary" } else { "secondary" },);

        let mut characteristics: Vec<_> = service.characteristics.into_iter().collect();
        characteristics
            .sort_by(|left, right| left.uuid.as_hyphenated().cmp(right.uuid.as_hyphenated()));

        if characteristics.is_empty() {
            println!("    (no characteristics)");
            continue;
        }

        for characteristic in characteristics {
            println!("    {} {:?}", characteristic.uuid, characteristic.properties);
        }
    }
}

#[cfg(feature = "btleplug-support")]
fn print_device_status(status: &iqos::DeviceStatus) {
    println!("Stick firmware: {}", status.stick_firmware);
    match &status.holder_firmware {
        Some(fw) => println!("Holder firmware: {fw}"),
        None => println!("Holder firmware: n/a (no holder support)"),
    }
    match status.battery_voltage {
        Some(v) => println!("Battery voltage: {v:.3} V"),
        None => println!("Battery voltage: read failed"),
    }
}

#[cfg(feature = "btleplug-support")]
fn print_device_info(info: &iqos::DeviceInfo) {
    println!("  model number: {}", info.model_number.as_deref().unwrap_or("(missing)"));
    println!("  serial number: {}", info.serial_number.as_deref().unwrap_or("(missing)"));
    println!("  software revision: {}", info.software_revision.as_deref().unwrap_or("(missing)"));
    println!("  manufacturer: {}", info.manufacturer_name.as_deref().unwrap_or("(missing)"));
}

#[cfg(test)]
mod tests {
    use super::{CliArgs, Command, ProbeCommand, parse_args};

    fn parse(arguments: &[&str]) -> Result<CliArgs, String> {
        parse_args(arguments.iter().map(|value| (*value).to_string()))
    }

    #[test]
    fn parses_inspect_command_without_filter() {
        let args = parse(&["iqos", "inspect"]).expect("inspect should parse");

        assert_eq!(args, CliArgs { name_filter: None, command: Command::Inspect });
    }

    #[test]
    fn parses_probe_command_with_name_filter() {
        let args = parse(&["iqos", "--name", "prime", "probe", "firmware-stick"])
            .expect("probe command should parse");

        assert_eq!(
            args,
            CliArgs {
                name_filter: Some("prime".to_string()),
                command: Command::Probe(ProbeCommand::FirmwareStick),
            }
        );
    }

    #[test]
    fn rejects_unknown_probe_subcommand() {
        let error = parse(&["iqos", "probe", "unknown"]).expect_err("unknown probe should fail");

        assert!(error.contains("unknown probe subcommand"));
    }

    #[test]
    fn rejects_missing_name_value() {
        let error = parse(&["iqos", "--name"]).expect_err("missing filter should fail");

        assert_eq!(error, "missing value for --name");
    }

    #[test]
    fn rejects_invalid_command_shape() {
        let error = parse(&["iqos", "inspect", "extra"]).expect_err("extra args should fail");

        assert_eq!(error, "invalid command shape");
    }
}
