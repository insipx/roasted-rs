//! Background runner task for embassy radio wifi interface.
//! Also handles TCP and UDP socket connections for Zippy

use alloc::string::String;
use core::time::Duration;

use anyhow::{Result, anyhow, bail};
use backon::{ExponentialBuilder, Retryable};
use embassy_futures::select;
use embassy_net::{Stack, dns::DnsQueryType, tcp::TcpSocket};
use url::Url;

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
    let mut session = setup_channels(&db, stack, &mut rx, &mut tx).await?;
    tracing::info!(addr = ?session.remote_endpoint(), "tcp connected");
    loop {
        match subscriber.next_message_pure().await {
            Events::SetDaemonUrl => {
                // TODO: should do a ordered shutdown of socket close etc.
                drop(session);
                session = setup_channels(&db, &stack, &mut rx, &mut tx).await?;
            }
            _ => continue,
        };
    }
    Ok(())
}

async fn setup_channels<'a>(
    db: &DbReader<String>,
    stack: &Stack<'static>,
    rx: &'a mut [u8],
    tx: &'a mut [u8],
) -> Result<TcpChannel<'a>> {
    let url = db.daemon_url().await.ok_or(anyhow!("no daemon url set"))?;
    let (ip, port) = find_ip_and_port(url.parse()?, stack).await?;
    let tcp = TcpSocket::new(*stack, rx, tx);
    let tcp_channel =
        TcpChannel::builder().stack(*stack).ip(&ip).port(port).socket(tcp).connect().await?;
    Ok(tcp_channel)
}

/// get ip and port
async fn find_ip_and_port(
    url: Url,
    stack: &Stack<'static>,
) -> Result<(embassy_net::IpAddress, u16)> {
    let host = url.host_str().ok_or_else(|| anyhow!("no host specified"))?;
    let port = url.port_or_known_default().ok_or_else(|| anyhow!("no port specified"))?;
    // TODO: should try all ips from dns query and if not retry/backoff (implement retry/backoff
    // in `tcp_channel` top-level fn)
    tracing::info!(host, "querying dns");
    let mut ips = stack
        .dns_query(host, DnsQueryType::A)
        .await
        .map_err(|e| anyhow!("failed to lookup ip address for {host}: {e:?}"))?;
    print_ips(&ips);
    if ips.is_empty() {
        bail!("no IP addresses returned for {host}")
    }
    Ok((ips.remove(0), port))
}

fn print_ips(ips: &[embassy_net::IpAddress]) {
    for ip in ips {
        tracing::info!(%ip, "found ip");
    }
}
