use core::{
    io,
    net::SocketAddr,
    task::{Context, Poll, ready},
};

use tokio::{io::ReadBuf, net::UdpSocket};

use super::UdpSocket as UdpSocketT;

pub struct UdpSocketPeer {
    socket: tokio::net::UdpSocket,
    peer: SocketAddr,
}

impl UdpSocketPeer {
    pub fn new(socket: UdpSocket, peer: SocketAddr) -> Self {
        Self { socket, peer }
    }
}

impl UdpSocketT for UdpSocketPeer {
    fn poll_recv(&self, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        let mut buf = ReadBuf::new(buf);
        let peer = ready!(self.socket.poll_recv_from(cx, &mut buf))?;
        if peer == self.peer {
            return Poll::Ready(Ok(buf.filled().len()));
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }

    fn poll_send(&self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        self.socket.poll_send_to(cx, buf, self.peer)
    }
}
