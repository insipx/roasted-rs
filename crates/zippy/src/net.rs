//! Background runner task for embassy radio wifi interface.
//! Also handles TCP and UDP socket connections for Zippy

use alloc::{boxed::Box, ffi::CString, format, string::String};
use core::{ffi::CStr, time::Duration};

use anyhow::{Context, Result, anyhow, bail};
use backon::{ExponentialBuilder, RetryableWithContext};
use embassy_futures::select::{Either, select};
use embassy_net::{
    IpAddress, IpEndpoint, Stack,
    dns::DnsQueryType,
    tcp::TcpSocket,
    udp::{PacketMetadata, UdpSocket},
};
use esp_hal::rng::Trng;
use futures_util::sink::SinkExt;
use roasted_data_streams::udp::{
    AudioPacketHeader, AudioPacketRef, UdpDtlsSocket, embedded_net::EmbassyPeerSocket,
};
use url::Url;

use crate::{
    audio::IngestReceiver,
    configuration::{DEFAULT_DAEMON_UDP_PORT, DTLS_ROOT_CA},
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

struct Args<'a> {
    stack: &'a Stack<'static>,
    db: &'a DbReader<String>,
    audio_ingest: &'a mut IngestReceiver<'static>,
    attempts: usize,
}

#[embassy_executor::task]
pub async fn net(
    stack: Stack<'static>,
    db: DbReader<String>,
    mut audio_ingest: IngestReceiver<'static>,
) {
    tracing::info!("starting TCP/UDP channels");
    let args = Args { stack: &stack, db: &db, audio_ingest: &mut audio_ingest, attempts: 0 };

    let (_, task) = (async |mut args| {
        let result = run(&mut args).await;
        args.attempts += 1;
        (args, result)
    })
    .retry(ExponentialBuilder::default().without_max_times())
    .sleep(backon::EmbassySleeper)
    .context(args)
    .notify(|err: &anyhow::Error, dur: Duration| {
        tracing::info!("encountered `{err}` retrying after {dur:?}");
    })
    .await;
    if let Err(e) = task {
        panic!("unrecoverable error occured in net task {e}");
    }
}

async fn run(args: &mut Args<'_>) -> Result<()> {
    let mut subscriber = crate::eventbus::subscriber()?;
    let Args { stack, db, audio_ingest, .. } = args;
    let (mut rx, mut tx) = ([0u8; 128], [0u8; 128]);
    let mut udp_channels = UdpChannels::new();
    stack.wait_config_up().await;
    let mut channels = Channels::builder().stack(stack.clone()).udp_channels(udp_channels);
    let url: Url = db.daemon_url().await.ok_or(anyhow!("no daemon url set"))?.parse()?;
    let udp_port: u16 = db.udp_port().await.unwrap_or(DEFAULT_DAEMON_UDP_PORT.into()).parse()?;
    let (ip, port) = find_ip_and_port(&url, stack).await?;
    let mut host = CString::new(url.host_str().ok_or(anyhow!("no host"))?).expect("CString failed");
    let mut channels = channels.ip(ip).tcp_port(port).udp_port(udp_port).udp_host(host).build();
    let (mut session, mut udp_session) = setup_channels(&mut channels).await?;

    loop {
        match select(audio_ingest.receive(), subscriber.next_message_pure()).await {
            Either::First(audio) => {
                tracing::info!("sending audio packet");
                udp_session
                    .send(AudioPacketRef {
                        header: AudioPacketHeader { sequence: audio.sequence, stream_id: 0 },
                        pcm: audio.pcm.as_slice(),
                    })
                    .await?;
                audio_ingest.receive_done();
                tracing::info!("sent audio packet");
            }
            Either::Second(command) => match command {
                Events::SetDaemonUrl => {
                    // timeout
                    embassy_time::with_timeout(
                        embassy_time::Duration::from_millis(1_000),
                        SinkExt::close(&mut udp_session),
                    )
                    .await
                    .map_err(|_| anyhow!("UDP socket close timed out"))??;
                    session.shutdown().await?;
                    drop(udp_session);
                    let url: Url =
                        db.daemon_url().await.ok_or(anyhow!("no daemon url set"))?.parse()?;
                    let (ip, port) = find_ip_and_port(&url, stack).await?;
                    host = CString::new(url.host_str().ok_or(anyhow!("no host"))?)
                        .expect("CString failed");
                    channels.new_addr(ip, Some(port), None, host);
                    (session, udp_session) = setup_channels(&mut channels).await?;
                }
                _ => continue,
            },
        }
    }
    Ok(())
}

async fn setup_channels<'a>(
    channels: &'a mut Channels,
) -> Result<(TcpChannel<'a>, UdpDtlsSocket<'a>)> {
    let (r1, r2) = embassy_futures::join::join(
        setup_tcp(
            channels.ip,
            channels.tcp_port,
            &channels.stack,
            &mut channels.rx,
            &mut channels.tx,
        ),
        setup_udp(
            channels.ip,
            channels.udp_port,
            channels.udp_host.as_c_str(),
            &channels.stack,
            &mut channels.udp_channels,
        ),
    )
    .await;
    let tcp = r1?;
    let udp = r2?;
    Ok((tcp, udp))
}

async fn setup_tcp<'a>(
    ip: IpAddress,
    port: u16,
    stack: &Stack<'static>,
    rx: &'a mut [u8],
    tx: &'a mut [u8],
) -> Result<TcpChannel<'a>> {
    let mut tcp = TcpSocket::new(*stack, rx, tx);
    tcp.set_keep_alive(Some(embassy_time::Duration::from_secs(5)));
    tcp.set_timeout(Some(embassy_time::Duration::from_secs(10)));
    let tcp_channel =
        TcpChannel::builder().stack(*stack).ip(&ip).port(port).socket(tcp).connect().await?;
    tracing::info!(addr = ?tcp_channel.remote_endpoint(), "tcp connected");
    Ok(tcp_channel)
}

async fn setup_udp<'a>(
    ip: IpAddress,
    port: u16,
    host: &CStr,
    stack: &Stack<'static>,
    channels: &'a mut UdpChannels,
) -> Result<UdpDtlsSocket<'a>> {
    let mut socket = UdpSocket::new(
        *stack,
        &mut channels.rx_meta,
        &mut channels.rx,
        &mut channels.tx_meta,
        &mut channels.tx,
    );
    socket.bind(0u16).map_err(|_| anyhow!("unable to bind to ephemeral port for udp"))?;
    let peer = EmbassyPeerSocket::new(socket, IpEndpoint { addr: ip, port });
    let rng =
        Trng::try_new().map_err(|err| anyhow!("unable to initialize RNG for DTLS: {err:?}"))?;
    let mut udp_channel = UdpDtlsSocket::new_client(Box::new(peer), rng, DTLS_ROOT_CA, host)
        .with_context(|| format!("failed creating new DTLS Socket"))?;
    tracing::info!(host = &*host.to_string_lossy(), port, "UDP connected");
    udp_channel.handshake().await?;
    tracing::debug!("DTLS Handshake success");
    Ok(udp_channel)
}

/// get ip and port
async fn find_ip_and_port(
    url: &Url,
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
