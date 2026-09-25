use color_eyre::eyre::Result;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::args::Args;

mod args;

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
        tracing_subscriber::registry().with(fmt::layer()).init();
    }

    println!("Hello, world! {}", gaggimate);
    Ok(())
}
