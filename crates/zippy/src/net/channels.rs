use alloc::format;

use anyhow::{Context, Result, anyhow, bail};
use embassy_net::{
    IpAddress, IpEndpoint, Stack, dns::DnsQueryType, tcp::TcpSocket, udp::UdpSocket,
};
use url::Url;

use super::*;

pub struct TcpChannel<'bufs> {
    stack: Stack<'static>,
    ip: IpAddress,
    port: u16,
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
        Ok(Self { stack, ip: *ip, port, socket })
    }

    pub fn remote_endpoint(&self) -> Option<IpEndpoint> {
        self.socket.remote_endpoint()
    }
    // todo: close the socket, or add fn for reopining to diff url
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
