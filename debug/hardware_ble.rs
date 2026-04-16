use std::io;

use iqos::Iqos;

#[path = "hardware_ble/connect.rs"]
mod connect;
#[path = "hardware_ble/exercises.rs"]
mod exercises;
#[path = "hardware_ble/suites.rs"]
mod suites;

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(crate) struct Config {
    pub(crate) name_filter: Option<String>,
    pub(crate) stateful_writes: bool,
    pub(crate) vibrate_millis: u64,
}

fn load_config() -> Config {
    Config {
        name_filter: std::env::var("IQOS_TEST_NAME_SUBSTRING").ok().filter(|s| !s.is_empty()),
        stateful_writes: std::env::var("IQOS_TEST_ALLOW_STATEFUL_WRITES")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        vibrate_millis: std::env::var("IQOS_TEST_VIBRATE_MILLIS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(500),
    }
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("\nfatal: {error}");
        std::process::exit(1);
    }
}

async fn run() -> TestResult {
    let config = load_config();
    println!("=== IQOS Hardware BLE Harness ===\n");

    let filter_hint =
        config.name_filter.as_deref().map(|f| format!(" (filter: \"{f}\")")).unwrap_or_default();
    println!("Connecting{filter_hint}...");

    let session = connect::scan_and_connect(config.name_filter.as_deref()).await.map_err(|e| {
        eprintln!("[❌] Connection failed: {e}");
        e
    })?;

    println!("[✅] Connected  model={:?}\n", session.model());

    let iqos = Iqos::new(session.clone());
    let mut failed: u32 = 0;

    macro_rules! suite {
        ($name:literal, $call:expr) => {{
            println!("--- {} ---", $name);
            match $call {
                Ok(_) => println!("  → passed\n"),
                Err(e) => {
                    println!("  → FAILED: {e}\n");
                    failed += 1;
                }
            }
        }};
    }

    suite!("Snapshot", suites::snapshot(&session, &iqos).await);

    if config.stateful_writes {
        suite!("Exercises", suites::exercise_all(&session, &iqos, config.vibrate_millis).await);
    } else {
        println!(
            "--- Exercises ---\n  \
             (skipped — set IQOS_TEST_ALLOW_STATEFUL_WRITES=1 to enable)\n"
        );
    }

    if failed == 0 {
        println!("=== all suites passed ===");
        Ok(())
    } else {
        println!("=== {failed} suite(s) failed ===");
        Err(io::Error::other(format!("{failed} suite(s) failed")).into())
    }
}
