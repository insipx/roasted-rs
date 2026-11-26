use alloc::format;

use anyhow::{Context, Result, anyhow, bail};
use embassy_net::{
    IpAddress, IpEndpoint, Stack, dns::DnsQueryType, tcp::TcpSocket, udp::UdpSocket,
};
use url::Url;

use super::*;

#[derive(bon::Builder)]
pub struct Channels {
    pub stack: Stack<'static>,
    pub ip: IpAddress,
    pub tcp_port: u16,
    #[builder(default = [0u8; 128])]
    pub rx: [u8; 128],
    #[builder(default = [0u8; 128])]
    pub tx: [u8; 128],
    pub udp_port: u16,
    pub udp_channels: UdpChannels,
    pub udp_host: CString,
}

impl Channels {
    pub fn new_addr(
        &mut self,
        ip: IpAddress,
        tcp_port: Option<u16>,
        udp_port: Option<u16>,
        udp_host: CString,
    ) {
        self.ip = ip;
        self.tcp_port = tcp_port.unwrap_or(self.tcp_port);
        self.udp_port = udp_port.unwrap_or(self.udp_port);
        self.udp_host = udp_host;
    }
}

pub struct TcpChannel<'bufs> {
    socket: TcpSocket<'bufs>,
}

#[bon::bon]
impl<'bufs> TcpChannel<'bufs> {
    #[builder(start_fn = builder, finish_fn = connect)]
    pub async fn connect(
        ip: &IpAddress,
        port: u16,
        mut socket: TcpSocket<'bufs>,
        stack: Stack<'static>,
    ) -> Result<Self> {
        let addr = IpEndpoint { addr: *ip, port };
        socket
            .connect(addr)
            .await
            .with_context(|| format!("failed to connect to tcp://{ip}:{port}"))?;
        Ok(Self { socket })
    }

    pub fn remote_endpoint(&self) -> Option<IpEndpoint> {
        self.socket.remote_endpoint()
    }

    pub async fn shutdown(mut self) -> Result<()> {
        self.socket.close();
        embassy_time::with_timeout(embassy_time::Duration::from_millis(1_000), self.socket.flush())
            .await
            .map_err(|_| anyhow!("TCP socket close timed out"))??;
        Ok(())
    }
}

pub(super) struct UdpChannels {
    pub(super) rx_meta: [PacketMetadata; 4],
    pub(super) tx_meta: [PacketMetadata; 4],
    pub(super) rx: [u8; 4096],
    pub(super) tx: [u8; 4096],
}

impl UdpChannels {
    pub(super) fn new() -> Self {
        Self {
            rx_meta: [PacketMetadata::EMPTY; 4],
            tx_meta: [PacketMetadata::EMPTY; 4],
            rx: [0u8; 4096],
            tx: [0u8; 4096],
        }
    }
}
