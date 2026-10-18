use anyhow::Result;
use embassy_net::{Stack, tcp::TcpSocket};

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
pub async fn tcp_channel(stack: Stack<'static>) {
    tracing::info!("starting TCP channel");
    if let Err(e) = run(stack).await {
        panic!("TCP Channel Task Failed {e:?}");
    }
}

async fn run(stack: Stack<'static>) -> Result<()> {
    let (mut rx, mut tx) = ([0u8; 128], [0u8; 128]);
    let tcp = TcpSocket::new(stack, &mut rx, &mut tx);
    stack.wait_config_up().await;

    let temp_url = "ws://www.example.com/ws".parse().unwrap();
    let session = TcpChannel::builder().stack(stack).url(temp_url).socket(tcp).connect().await?;
    Ok(())
}
