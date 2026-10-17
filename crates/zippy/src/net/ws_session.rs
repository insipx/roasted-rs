use core::net::SocketAddr;

use anyhow::{Result, anyhow, bail};
use crypto_common::rand_core::Rng;
use edge_http::{
    io::client::Connection,
    ws::{MAX_BASE64_KEY_LEN, MAX_BASE64_KEY_RESPONSE_LEN, NONCE_LEN},
};
use edge_nal_embassy::{Tcp, TcpSocket};
use embassy_net::{Stack, dns::DnsQueryType};
use esp_hal::rng::Trng;
use esp_println::println;
use url::Url;

pub struct WebSocketSession<'buf> {
    stack: Stack<'static>,
    url: Url,
    socket: TcpSocket<'buf>,
    buf: &'buf mut [u8],
}

#[bon::bon]
impl<'buf> WebSocketSession<'buf> {
    #[builder(start_fn = builder, finish_fn = connect)]
    pub async fn connect(
        stack: Stack<'static>,
        url: Url,
        buf: &'buf mut [u8],
        tcp: &'buf Tcp<'_>,
    ) -> Result<Self> {
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
        /*
                let mut connection: Connection<Tcp> =
                    Connection::new(buf, &tcp, SocketAddr::new((*ip).into(), port));

                let mut nonce = [0_u8; NONCE_LEN];
                rng.fill_bytes(&mut nonce);

                let mut buf = [0_u8; MAX_BASE64_KEY_LEN];
                connection
                    .initiate_ws_upgrade_request(Some(host), None, path, None, &nonce, &mut buf)
                    .await?;
                connection.initiate_response().await?;

                let mut buf = [0_u8; MAX_BASE64_KEY_RESPONSE_LEN];
                if !connection.is_ws_upgrade_accepted(&nonce, &mut buf)? {
                    bail!("WS upgrade failed");
                }

                connection.complete().await?;

                // Now we have the TCP socket in a state where it can be operated as a WS connection
                // Send some traffic to a WS echo server and read it back
                let (socket, buf) = connection.release();
        */
        println!("Connection upgraded to WS");
        Ok(Self { stack, url, socket, buf })
    }
}
