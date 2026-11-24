use std::{
    ffi::CStr,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use color_eyre::eyre::{Result, WrapErr};
use futures::Stream;
use getrandom::SysRng;
use pin_project_lite::pin_project;
use roasted_data_streams::udp::{AudioPacketHeader, UdpDtlsSocket, UdpSocketPeer};
use tokio::net::UdpSocket;

pin_project! {
    pub struct AudioListener {
        #[pin]
        inner: UdpDtlsSocket<'static>,
    }
}

impl AudioListener {
    /// Create a DTLS/UDP audio listener bound to a single Zippy peer.
    pub async fn listen(
        address: SocketAddr,
        certificate_chain: &CStr,
        private_key: &CStr,
    ) -> Result<Self> {
        let socket = UdpSocket::bind(address)
            .await
            .wrap_err_with(|| format!("audio listener failed binding to {address}"))?;

        let peer = socket.peek_sender().await.wrap_err("no audio peer")?;
        let transport = UdpSocketPeer::new(socket, peer);
        let inner = UdpDtlsSocket::new_server(
            Box::new(transport),
            SysRng,
            peer,
            certificate_chain,
            private_key,
        )
        .wrap_err_with(|| format!("failed DTLS setup for {peer}"))?;

        Ok(Self { inner })
    }
}

impl Stream for AudioListener {
    type Item = Result<(AudioPacketHeader, Box<[u8]>)>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx).map(|item| item.map(|packet| packet.map_err(Into::into)))
    }
}
