use alloc::string::String;

use anyhow::{Result, anyhow};
use embassy_net::{Stack, tcp::TcpSocket};

use crate::db::{self, DbReader};

mod tcp_channel;
mod wifi;
pub use tcp_channel::*;
pub use wifi::*;

#[embassy_executor::task]
pub async fn bg_runner(mut runner: embassy_net::Runner<esp_radio::wifi::Interface, 'static>) {
    runner.run().await;
}

// TODO: add retries and backoff strategy
#[embassy_executor::task]
pub async fn tcp_channel(stack: Stack<'static>, db: DbReader<String>) {
    tracing::info!("starting TCP channel");
    if let Err(e) = run(stack, db).await {
        panic!("TCP Channel Task Failed {e:?}");
    }
}

async fn run(stack: Stack<'static>, db: DbReader<String>) -> Result<()> {
    let (mut rx, mut tx) = ([0u8; 128], [0u8; 128]);
    let tcp = TcpSocket::new(stack, &mut rx, &mut tx);
    stack.wait_config_up().await;
    let read = db.read().await;

    let url = read.get(&db::Keys::DURL).ok_or(anyhow!("no daemon url set"))?;
    let session =
        TcpChannel::builder().stack(stack).url(url.parse()?).socket(tcp).connect().await?;
    Ok(())
}
