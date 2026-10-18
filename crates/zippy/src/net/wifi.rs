//! WiFi Task manages WiFi

use alloc::string::String;

use anyhow::{Result, anyhow};
use esp_radio::wifi::{AuthenticationMethodConfig, Config, WifiController, sta::StationConfig};

use crate::{db, eventbus::Events, net::DbReader};

#[embassy_executor::task]
pub async fn wifi(controller: WifiController<'static>, db: DbReader<String>) {
    tracing::info!("starting WiFi controller");
    if let Err(e) = run(controller, db).await {
        panic!("WiFi Controller task failed {e:?}");
    }
}

async fn run(mut controller: WifiController<'_>, db: DbReader<String>) -> Result<()> {
    let mut subscriber = crate::eventbus::subscriber()?;
    loop {
        let _ = match subscriber.next_message_pure().await {
            Events::SetWiFi | Events::Start => (),
            _ => continue,
        };
        let db = db.read().await;
        let ssid = db.get(&db::Keys::SSID).ok_or(anyhow!("no ssid set"))?;
        let pw = db.get(&db::Keys::WIPW).ok_or(anyhow!("no wifi password set"))?;
        tracing::info!(ssid, "connecting to WiFi network");
        let config = Config::Station(
            StationConfig::default().with_ssid(ssid.as_str().try_into()?).with_authentication(
                AuthenticationMethodConfig::Wpa2Personal(pw.as_str().try_into()?),
            ),
        );
        controller.set_config(&config)?;
        controller.connect_async().await?;
    }
}
