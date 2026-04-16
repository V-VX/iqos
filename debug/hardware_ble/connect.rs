use btleplug::api::{Central as _, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use iqos::IqosBle;
use tokio::time::{Duration, sleep};

use crate::TestResult;

const SCAN_SECONDS: u64 = 5;

pub(crate) async fn scan_and_connect(name_filter: Option<&str>) -> TestResult<IqosBle> {
    let adapter = first_adapter().await?;
    println!("  Scanning {} s...", SCAN_SECONDS);
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
    let normalized_filter = name_filter.map(|s| s.to_ascii_lowercase());
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
            .is_some_and(|f| !name.to_ascii_lowercase().contains(f.as_str()))
        {
            continue;
        }
        candidates.push((name, peripheral));
    }

    candidates.sort_by(|a, b| a.0.cmp(&b.0));
    candidates.into_iter().next().map(|(_, p)| p).ok_or_else(|| {
        let hint = name_filter.map(|f| format!(" matching \"{f}\"")).unwrap_or_default();
        format!("no IQOS device found{hint}").into()
    })
}
