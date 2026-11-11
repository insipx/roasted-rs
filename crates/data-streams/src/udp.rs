//! Types for UDP Streams
use core::{
    mem::MaybeUninit,
    net::SocketAddr,
    task::{Context, Poll, Waker},
};

use mbedtls_rs_sys::{
    MBEDTLS_SSL_IS_CLIENT, MBEDTLS_SSL_PRESET_DEFAULT, MBEDTLS_SSL_TRANSPORT_DATAGRAM,
    mbedtls_ssl_config, mbedtls_ssl_config_defaults, mbedtls_ssl_config_init,
};

mod embedded_net;
mod tokio;

pub trait UdpSocket {
    type Error;

    fn poll_recv_from(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<SocketAddr, Self::Error>>;
}

pub struct UdpDtlsSocket<S: UdpSocket> {
    context: DtlsWakerContext<S>,
    ssl_config: Box<mbedtls_ssl_config>,
}

impl<S: UdpSocket> UdpDtlsSocket<S> {
    pub fn new(socket: S) -> Self {
        let mut ssl_config = Box::new(MaybeUninit::<mbedtls_ssl_config>::uninit());
        let mut ssl_config = unsafe {
            mbedtls_ssl_config_init(ssl_config.as_mut_ptr());
            ssl_config.assume_init()
        };
        // TODO: handle i32 C return value (need result type)
        unsafe {
            mbedtls_ssl_config_defaults(
                ssl_config.as_mut(),
                MBEDTLS_SSL_IS_CLIENT as i32,
                MBEDTLS_SSL_TRANSPORT_DATAGRAM as i32,
                MBEDTLS_SSL_PRESET_DEFAULT as i32,
            );
        }
        Self { context: DtlsWakerContext { waker: None, socket }, ssl_config }
    }
}

pub struct DtlsWakerContext<S> {
    waker: Option<Waker>,
    socket: S,
}
// need to decide on framing
// will be cobs framed struct first
// then # of bytes to read that is the audio
impl<S> Stream for UdpTlsSocket<S> {
    type Item = 
}

// DTLS Streams
//
//Executor
//  │
//  ▼
//UdpDtlsStream::poll_next(&mut cx)
//  │ Save cx.waker().clone() in callback state
//  ▼
//mbedtls_ssl_read(...)
//  │ Needs encrypted UDP data
//  ▼
//recv_callback(state, buf, len)
//  │ Reconstruct Context from saved waker
//  ▼
//UdpSocket::poll_recv_from(buf, &mut cx)
//  │
//  ├── Packet available
//  │     ▼
//  │   Callback returns byte count
//  │     ▼
//  │   Mbed TLS decrypts/authenticates
//  │     ▼
//  │   poll_next returns Ready(Some(Ok(message)))
//  │   [or continues if more protocol work is needed]
//  │
//  └── No packet
//        ▼
//      Embassy registers waker, returns Pending
//        ▼
//      Callback returns WANT_READ
//        ▼
//      Mbed TLS returns WANT_READ
//        ▼
//      poll_next returns Pending
//        │
//        │ Later: packet arrives (or DTLS timer expires)
//        ▼
//      Waker wakes executor → poll_next runs again
//
