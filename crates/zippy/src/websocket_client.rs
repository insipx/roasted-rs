use core::net::SocketAddr;

use anyhow::{Result, anyhow, bail};
use crypto_common::rand_core::Rng;
use edge_http::{
    io::client::Connection,
    ws::{MAX_BASE64_KEY_LEN, MAX_BASE64_KEY_RESPONSE_LEN, NONCE_LEN},
};
use edge_nal_embassy::{Tcp, TcpBuffers};
use embassy_net::{Stack, dns::DnsQueryType};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Trng;
use esp_println::println;
use url::Url;

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<esp_radio::wifi::Interface, 'static>) {
    runner.run().await;
}

#[embassy_executor::task]
pub async fn ws_task(stack: Stack<'static>) {
    let mut buf = [0u8; 8192]; // 8kb  scratch (necessary?)
    loop {
        stack.wait_config_up().await;

        let temp_url = "ws://www.example.com/ws".parse().unwrap();
        if let Err(error) = websocket_session(stack, temp_url, &mut buf).await {
            esp_println::println!("WebSocket disconnected: {error:?}");
        }

        Timer::after(Duration::from_secs(3)).await;
    }
}

async fn websocket_session(stack: Stack<'static>, url: Url, buf: &mut [u8]) -> Result<()> {
    let mut rng = Trng::try_new().expect("RNG must be active");

    if url.scheme() != "ws" {
        bail!("only ws:// URLs are supported");
    }
    let host = url.host_str().ok_or_else(|| anyhow!("no host specified"))?;
    let port = url.port_or_known_default().ok_or_else(|| anyhow!("no port specified"))?;
    let path = url.path();
    let ips = stack
        .dns_query(host, DnsQueryType::A)
        .await
        .map_err(|e| anyhow!("failed to lookup ip address for {host}: {e:?}"))?;
    let ip = ips.first().ok_or_else(|| anyhow!("no IP addresses returned for {host}"))?;
    let tcp_buffers = TcpBuffers::<1, 1024, 1024>::new();
    let tcp = Tcp::new(stack, &tcp_buffers);
    let mut conn: Connection<_> = Connection::new(buf, &tcp, SocketAddr::new((*ip).into(), port));

    let mut nonce = [0_u8; NONCE_LEN];
    rng.fill_bytes(&mut nonce);

    let mut buf = [0_u8; MAX_BASE64_KEY_LEN];
    conn.initiate_ws_upgrade_request(Some(host), None, path, None, &nonce, &mut buf).await?;
    conn.initiate_response().await?;

    let mut buf = [0_u8; MAX_BASE64_KEY_RESPONSE_LEN];
    if !conn.is_ws_upgrade_accepted(&nonce, &mut buf)? {
        bail!("WS upgrade failed");
    }

    conn.complete().await?;

    // Now we have the TCP socket in a state where it can be operated as a WS connection
    // Send some traffic to a WS echo server and read it back

    let (_socket, _buf) = conn.release();

    println!("Connection upgraded to WS, starting traffic now");
    todo!()
}
