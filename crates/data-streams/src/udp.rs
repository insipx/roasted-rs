//! Types for UDP Streams
use alloc::{boxed::Box, vec::Vec};
use core::{
    cell::RefCell,
    ffi::{c_int, c_void},
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures_core::Stream;
use mbedtls_rs_sys::{
    MBEDTLS_ERR_SSL_INTERNAL_ERROR, MBEDTLS_ERR_SSL_PEER_CLOSE_NOTIFY, MBEDTLS_ERR_SSL_WANT_READ,
    MBEDTLS_ERR_SSL_WANT_WRITE, MBEDTLS_SSL_IN_CONTENT_LEN, MBEDTLS_SSL_IS_CLIENT,
    MBEDTLS_SSL_PRESET_DEFAULT, MBEDTLS_SSL_TRANSPORT_DATAGRAM, mbedtls_ssl_conf_rng,
    mbedtls_ssl_config, mbedtls_ssl_config_defaults, mbedtls_ssl_config_init, mbedtls_ssl_context,
    mbedtls_ssl_init, mbedtls_ssl_read, mbedtls_ssl_set_bio, mbedtls_ssl_setup,
};
use rand_core::TryCryptoRng;
use serde::{Deserialize, Serialize};

use crate::error::{DtlsReadError, DtlsRngError};

#[cfg(feature = "embassy-net")]
pub mod embedded_net;
#[cfg(feature = "std")]
mod tokio;

mod mbedtls_callbacks;
mod sink;
mod stream;
#[cfg(any(feature = "std", feature = "embassy-net"))]
mod timer;

pub use mbedtls_callbacks::*;
// pub use sink::*;
// pub use stream::*;

pub trait UdpSocket {
    fn poll_recv(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, core::io::Error>>;

    fn poll_send(&self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, core::io::Error>>;
}

pub trait DtlsRng {
    fn fill(&mut self, out: &mut [u8]) -> Result<(), DtlsRngError>;
}

impl<R: TryCryptoRng> DtlsRng for R {
    fn fill(&mut self, out: &mut [u8]) -> Result<(), DtlsRngError> {
        self.try_fill_bytes(out).map_err(|_| DtlsRngError)
    }
}

/// A UDP Socket that uses DTLS to communicate
/// _NOTE:_ MUST be owned/used by one task. If split into stream and sink parts
/// wakes will may be overwritten/missed.
// TODO: can use multiple wakers to make splitting into stream/sink safe.
pub struct UdpDtlsSocket<'net> {
    context: Box<RefCell<DtlsWakerContext<'net>>>,
    ssl_config: Box<mbedtls_ssl_config>,
    ssl_context: Box<mbedtls_ssl_context>,
    rng_context: Box<RefCell<Box<dyn DtlsRng + 'net>>>,
    // the constant size of the array here is way large for embedded (16kb) and should make it
    // smaller
    recv_buff: [u8; MBEDTLS_SSL_IN_CONTENT_LEN as usize],
    /// holds apending audio packet to be sent
    pending_send: Option<Vec<u8>>,
}

impl<'net> UdpDtlsSocket<'net> {
    /// On Tokio, poll this socket in a runtime with time enabled.
    pub fn new(socket: Box<dyn UdpSocket + 'net>, rng: impl TryCryptoRng + 'net) -> Self {
        Self::new_with_endpoint(socket, rng, MBEDTLS_SSL_IS_CLIENT as c_int)
    }

    fn new_with_endpoint(
        socket: Box<dyn UdpSocket + 'net>,
        rng: impl TryCryptoRng + 'net,
        endpoint: c_int,
    ) -> Self {
        let mut ssl_config = Box::new(MaybeUninit::<mbedtls_ssl_config>::uninit());
        let mut ssl_context = Box::new(MaybeUninit::<mbedtls_ssl_context>::uninit());
        let rng_context: Box<Rng<'net>> = Box::new(RefCell::new(Box::new(rng)));
        let mut ssl_config = unsafe {
            mbedtls_ssl_config_init(ssl_config.as_mut_ptr());
            ssl_config.assume_init()
        };
        let mut ssl_context = unsafe {
            mbedtls_ssl_init(ssl_context.as_mut_ptr());
            ssl_context.assume_init()
        };

        // TODO: handle i32 C return value (need result type)
        unsafe {
            mbedtls_ssl_config_defaults(
                ssl_config.as_mut(),
                endpoint,
                MBEDTLS_SSL_TRANSPORT_DATAGRAM as i32,
                MBEDTLS_SSL_PRESET_DEFAULT as i32,
            );
        }

        // need to setup rng provider before ssl setup
        let rng_ptr = core::ptr::from_ref(rng_context.as_ref()).cast_mut().cast();
        unsafe {
            mbedtls_ssl_conf_rng(&mut *ssl_config, Some(rng_callback), rng_ptr);
        }

        let result = unsafe { mbedtls_ssl_setup(&mut *ssl_context, &*ssl_config) };
        if result != 0 {
            panic!("error setting up mbedtls ssl");
        }
        let waker_context = Box::new(RefCell::new(DtlsWakerContext {
            waker: None,
            socket,
            #[cfg(any(feature = "std", feature = "embassy-net"))]
            delay: None,
        }));
        // set basic input/output for dtls
        unsafe {
            let context = core::ptr::from_ref(waker_context.as_ref()).cast_mut().cast();
            mbedtls_ssl_set_bio(
                &mut *ssl_context,
                context,
                Some(send_callback),
                Some(recv_callback),
                None,
            );
            #[cfg(any(feature = "std", feature = "embassy-net"))]
            mbedtls_rs_sys::mbedtls_ssl_set_timer_cb(
                &mut *ssl_context,
                context,
                Some(timer::set_delay),
                Some(timer::get_delay),
            );
        };
        Self {
            context: waker_context,
            ssl_config,
            ssl_context,
            rng_context,
            recv_buff: [0u8; MBEDTLS_SSL_IN_CONTENT_LEN as usize],
            pending_send: None,
        }
    }

    fn poll_timer(&self, cx: &mut Context<'_>) {
        #[cfg(any(feature = "std", feature = "embassy-net"))]
        if let Some(delay) = &mut self.context.borrow_mut().delay {
            delay.poll_wake(cx);
        }
    }
}

struct DtlsWakerContext<'net> {
    waker: Option<Waker>,
    socket: Box<dyn UdpSocket + 'net>,
    #[cfg(any(feature = "std", feature = "embassy-net"))]
    delay: Option<timer::Delay>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AudioPacketHeader {
    pub sequence: u16,
    pub stream_id: u8,
}

pub struct AudioPacketRef<'a> {
    pub header: AudioPacketHeader,
    pub pcm: &'a [u8],
}

pub type AudioPacket = AudioPacketRef<'static>;

pub type Rng<'net> = RefCell<Box<dyn DtlsRng + 'net>>;
