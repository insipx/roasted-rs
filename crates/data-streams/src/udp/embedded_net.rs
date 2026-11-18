use core::{
    io,
    task::{Context, Poll},
};

use embassy_net::{
    IpEndpoint,
    udp::{RecvError, UdpSocket as EmbassyUdpSocket},
};

use super::UdpSocket;

/// An adapter for embassy net which only receives datagrams from a single client socket addr
/// _NOTE_ this depends on peer being accurate which may be easily spoofed in udp.
/// this trait implementation is used in [UdpDtlsSocket] which initiates its own
/// cryptographic handshake providing some protection from spoofing.
pub struct EmbassyPeerSocket<'net> {
    socket: EmbassyUdpSocket<'net>,
    peer: IpEndpoint,
}

impl<'net> EmbassyPeerSocket<'net> {
    pub fn new(socket: EmbassyUdpSocket<'net>, peer: IpEndpoint) -> Self {
        Self { socket, peer }
    }
}

impl UdpSocket for EmbassyPeerSocket<'_> {
    fn poll_recv(&self, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        match self.socket.poll_recv_from(buf, cx) {
            Poll::Ready(Ok((count, meta))) if meta.endpoint == self.peer => Poll::Ready(Ok(count)),
            Poll::Ready(Ok(_)) | Poll::Ready(Err(RecvError::Truncated)) => {
                // discard this datagram. it is possible another datagram is queued
                // so wake again.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
