//! Background runner task for embassy radio wifi interface.
//! Also handles TCP and UDP socket connections for Zippy

use alloc::string::String;
use core::time::Duration;

use anyhow::{Result, anyhow};
use backon::{ExponentialBuilder, Retryable};
use embassy_futures::select;
use embassy_net::{Stack, tcp::TcpSocket};

use crate::{
    db::{self, DbReader},
    eventbus::Events,
};

mod channels;
mod wifi;
pub use channels::*;
pub use wifi::*;

#[embassy_executor::task]
pub async fn bg_runner(mut runner: embassy_net::Runner<esp_radio::wifi::Interface, 'static>) {
    runner.run().await;
}

#[derive(Copy, Clone)]
struct Args<'a> {
    stack: &'a Stack<'static>,
    db: &'a DbReader<String>,
}

// TODO: add retries and backoff strategy
#[embassy_executor::task]
pub async fn net(stack: Stack<'static>, db: DbReader<String>) {
    tracing::info!("starting TCP channel");
    let args = Args { stack: &stack, db: &db };

    let task = (|| async { run(args).await })
        .retry(ExponentialBuilder::default().without_max_times())
        .sleep(backon::EmbassySleeper)
        .notify(|err: &anyhow::Error, dur: Duration| {
            tracing::info!("encountered `{err}` retrying after {dur:?}");
        })
        .await;
    if let Err(e) = task {
        panic!("unrecoverable error occured in net task {e}");
    }
}

async fn run(args: Args<'_>) -> Result<()> {
    let mut subscriber = crate::eventbus::subscriber()?;
    let Args { stack, db } = args;
    let (mut rx, mut tx) = ([0u8; 128], [0u8; 128]);
    stack.wait_config_up().await;
    let mut tcp = TcpSocket::new(*stack, &mut rx, &mut tx);
    let url = db.daemon_url().await.ok_or(anyhow!("no daemon url set"))?;

    let mut session =
        TcpChannel::builder().stack(*stack).url(url.parse()?).socket(tcp).connect().await?;
    tracing::info!(addr = ?session.remote_endpoint(), "tcp connected");
    loop {
        match subscriber.next_message_pure().await {
            Events::SetDaemonUrl => {
                let url = db.daemon_url().await.ok_or(anyhow!("no daemon url set"))?;
                // TODO: should do a ordered shutdown of socket close etc.
                drop(session);
                let tcp = TcpSocket::new(*stack, &mut rx, &mut tx);
                session = TcpChannel::builder()
                    .stack(*stack)
                    .url(url.parse()?)
                    .socket(tcp)
                    .connect()
                    .await?;
            }
            _ => continue,
        };
    }
    Ok(())
}
