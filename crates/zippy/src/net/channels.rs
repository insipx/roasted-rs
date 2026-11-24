use anyhow::{Result, anyhow, bail};
use embassy_net::{
    IpAddress, IpEndpoint, Stack, dns::DnsQueryType, tcp::TcpSocket, udp::UdpSocket,
};
use url::Url;

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
        socket.connect(addr).await?;
        Ok(Self { stack, ip: *ip, port, socket })
    }

    pub fn remote_endpoint(&self) -> Option<IpEndpoint> {
        self.socket.remote_endpoint()
    }
    // todo: close the socket, or add fn for reopining to diff url
}

pub struct UdpChannel<'bufs> {
    stack: Stack<'static>,
    ip: IpAddress,
    socket: UdpSocket<'bufs>,
}

#[bon::bon]
impl<'bufs> UdpChannel<'bufs> {
    #[builder(start_fn = builder, finish_fn = connect)]
    pub async fn connect(ip: &IpAddress, stack: Stack<'static>) -> Result<Self> {
        todo!()
    }
}
