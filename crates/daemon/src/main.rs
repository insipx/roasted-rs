use std::path::PathBuf;

use color_eyre::eyre::Result;
use tracing::Level;
use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{args::Args, server::dtls::DtlsKeys};

mod args;
mod listeners;
mod psa;
mod server;

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { gaggimate, tcp_port, udp_port: _, loki } = args::parse_args()?;

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
    let keys = DtlsKeys::new()?;
    psa::init()?;
    let handle = tokio::task::spawn_local(server::tcp::server(
        tcp_port,
        gaggimate,
        PathBuf::from(db_url),
        keys,
    ));

    tokio::try_join!(handle)?.0?;
    Ok(())
}
