use std::path::PathBuf;

use color_eyre::eyre::Result;
use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::args::Args;

mod args;
mod listeners;
mod server;

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { gaggimate, ws_port, udp_port, loki } = args::parse_args()?;

    if let Some(loki_url) = loki {
        let (layer, task) = tracing_loki::builder()
            .label("host", "roasted-rs-daemon")?
            .extra_field("pid", format!("{}", std::process::id()))?
            .build_url(loki_url)?;
        tracing_subscriber::registry().with(layer).init();
        tokio::spawn(task);
    } else {
        tracing_subscriber::registry()
            .with(
                Targets::new()
                    .with_default(Level::INFO)
                    .with_target(env!("CARGO_PKG_NAME"), Level::TRACE),
            )
            .with(fmt::layer().compact().with_file(false))
            .init();
    }
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let handle =
        tokio::task::spawn_local(server::ws::server(ws_port, gaggimate, PathBuf::from(db_url)));

    tokio::try_join!(handle)?;
    // let mut s = GaggimateListener::connect(&gaggimate).await?;
    // while let Some(ev) = s.try_next().await? {
    //     println!("{:#?}", ev);
    // }
    Ok(())
}
