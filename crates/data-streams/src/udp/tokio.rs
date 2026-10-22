use core::{
    net::SocketAddr,
    task::{Context, Poll},
};

use tokio::{io::ReadBuf, net::UdpSocket};

use super::UdpSocket as UdpSocketT;

impl UdpSocketT for UdpSocket {
    type Error = core::io::Error;
    fn poll_recv_from(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<SocketAddr, Self::Error>> {
        let mut rbuf = ReadBuf::new(buf);
        self.poll_recv_from(cx, &mut rbuf)
    }
}
