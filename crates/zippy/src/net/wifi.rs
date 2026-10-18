//! WiFi Task manages WiFi

use alloc::{format, string::String};
use core::time::Duration;

use anyhow::{Context, Result, anyhow};
use backon::{ExponentialBuilder, RetryableWithContext};
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config, WifiController, ap::AccessPointInfo, scan::ScanConfig,
    sta::StationConfig,
};

use crate::{db, eventbus::Events, net::DbReader};

/// internal args for retries/run function
struct Args<'a> {
    controller: &'a mut WifiController<'static>,
    db: &'a DbReader<String>,
    attempt: usize,
}

impl Args<'_> {
    pub fn is_retry(&self) -> bool {
        self.attempt > 0
    }
}

#[embassy_executor::task]
pub async fn wifi(mut controller: WifiController<'static>, db: DbReader<String>) {
    tracing::info!("starting WiFi controller");
    let args = Args { controller: &mut controller, db: &db, attempt: 0 };
    let (_, task) = attempt
        .retry(ExponentialBuilder::default().without_max_times())
        .sleep(backon::EmbassySleeper)
        .context(args)
        .notify(|err: &anyhow::Error, dur: Duration| {
            tracing::info!("encountered `{err}` retrying after {dur:?}");
        })
        .await;
    if let Err(e) = task {
        panic!("unrecoverable error occured in WiFi task, {e}");
    }
}

async fn attempt<'a>(mut args: Args<'a>) -> (Args<'a>, Result<()>) {
    let result = run(&mut args).await;
    args.attempt += 1;
    (args, result)
}

async fn run(args: &mut Args<'_>) -> Result<()> {
    let mut subscriber = crate::eventbus::subscriber()?;
    loop {
        if !args.is_retry() {
            let _ = match subscriber.next_message_pure().await {
                Events::SetWiFi | Events::Start => (),
                _ => continue,
            };
        }
        let Args { controller, db, .. } = args;
        let scan_config = ScanConfig::default().with_max(10);
        let aps = controller.scan_async(&scan_config).await?;
        print_aps(&aps);

        let db = db.read().await;
        let ssid = db.get(&db::Keys::SSID).ok_or(anyhow!("no ssid set"))?;
        let pw = db.get(&db::Keys::WIPW).ok_or(anyhow!("no wifi password set"))?;
        tracing::info!(ssid, "connecting to WiFi network");
        let config = Config::Station(
            StationConfig::default().with_ssid(ssid.as_str().try_into()?).with_authentication(
                AuthenticationMethodConfig::Wpa2Personal(pw.as_str().try_into()?),
            ),
        );
        controller.set_config(&config).with_context(|| "failed to set WiFi config")?;
        controller.connect_async().await.with_context(|| format!("failed to connect to {ssid}"))?;
        tracing::info!(ssid, "connected to WiFi after {} tries", args.attempt);
        args.attempt = 0;
    }
}

fn print_aps(aps: &[AccessPointInfo]) {
    for ap in aps {
        tracing::info!(ssid = ap.ssid.as_str(), signal_strength = ap.signal_strength, auth = ?ap.auth_method, "Found WiFI Access Point");
    }
}
