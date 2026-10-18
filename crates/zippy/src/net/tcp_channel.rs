use anyhow::{Result, anyhow, bail};
use embassy_net::{IpEndpoint, Stack, dns::DnsQueryType, tcp::TcpSocket};
use esp_hal::rng::Trng;
use url::Url;

pub struct TcpChannel<'bufs> {
    stack: Stack<'static>,
    url: Url,
    socket: TcpSocket<'bufs>,
}

#[bon::bon]
impl<'bufs> TcpChannel<'bufs> {
    #[builder(start_fn = builder, finish_fn = connect)]
    pub async fn connect(
        url: Url,
        mut socket: TcpSocket<'bufs>,
        stack: Stack<'static>,
    ) -> Result<Self> {
        if url.scheme() != "tcp" {
            bail!("only tcp:// URLs (`tcp://${{host}}:${{port}}`) supported");
        }
        let host = url.host_str().ok_or_else(|| anyhow!("no host specified"))?;
        let port = url.port_or_known_default().ok_or_else(|| anyhow!("no port specified"))?;
        // TODO: should try all ips from dns query and if not retry/backoff (implement retry/backoff
        // in `tcp_channel` top-level fn)
        let ips = stack
            .dns_query(host, DnsQueryType::A)
            .await
            .map_err(|e| anyhow!("failed to lookup ip address for {host}: {e:?}"))?;
        let ip = ips.first().ok_or_else(|| anyhow!("no IP addresses returned for {host}"))?;
        let addr = IpEndpoint { addr: *ip, port };
        socket.connect(addr).await?;
        Ok(Self { stack, url, socket })
    }
}
