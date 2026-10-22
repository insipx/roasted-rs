//! Types for UDP Streams
use core::{
    mem::MaybeUninit,
    net::SocketAddr,
    task::{Context, Poll},
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
    inner: S,
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
        Self { inner: socket, ssl_config }
    }
}
