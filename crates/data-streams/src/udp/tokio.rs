use core::task::{Context, Poll};

use tokio::{io::ReadBuf, net::UdpSocket};

use super::UdpSocket as UdpSocketT;

impl UdpSocketT for UdpSocket {
    fn poll_recv(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, core::io::Error>> {
        let mut rbuf = ReadBuf::new(buf);
        core::task::ready!(self.poll_recv(cx, &mut rbuf))?;
        Poll::Ready(Ok(rbuf.filled().len()))
    }
}
