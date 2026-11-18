//! Types for UDP Streams
use alloc::boxed::Box;
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
    MBEDTLS_SSL_PRESET_DEFAULT, MBEDTLS_SSL_TRANSPORT_DATAGRAM, mbedtls_ssl_config,
    mbedtls_ssl_config_defaults, mbedtls_ssl_config_init, mbedtls_ssl_context, mbedtls_ssl_init,
    mbedtls_ssl_read, mbedtls_ssl_set_bio, mbedtls_ssl_setup,
};
use serde::{Deserialize, Serialize};

use crate::error::DtlsReadError;

#[cfg(feature = "embassy-net")]
pub mod embedded_net;
#[cfg(feature = "std")]
mod tokio;

pub trait UdpSocket {
    fn poll_recv(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, core::io::Error>>;
}

pub struct UdpDtlsSocket<'net> {
    context: Box<RefCell<DtlsWakerContext<'net>>>,
    ssl_config: Box<mbedtls_ssl_config>,
    ssl_context: Box<mbedtls_ssl_context>,
    // the constant size of the array here is way large for emberedded (16kb) and we should make it
    // smaller
    recv_buff: [u8; MBEDTLS_SSL_IN_CONTENT_LEN as usize],
}

impl<'net> UdpDtlsSocket<'net> {
    pub fn new(socket: Box<dyn UdpSocket + 'net>) -> Self {
        let mut ssl_config = Box::new(MaybeUninit::<mbedtls_ssl_config>::uninit());
        let mut ssl_context = Box::new(MaybeUninit::<mbedtls_ssl_context>::uninit());
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
                MBEDTLS_SSL_IS_CLIENT as i32,
                MBEDTLS_SSL_TRANSPORT_DATAGRAM as i32,
                MBEDTLS_SSL_PRESET_DEFAULT as i32,
            );
        }

        let result = unsafe { mbedtls_ssl_setup(&mut *ssl_context, &*ssl_config) };
        if result != 0 {
            panic!("error setting up mbedtls ssl");
        }
        let waker_context = Box::new(RefCell::new(DtlsWakerContext { waker: None, socket }));
        // set basic input/output for dtls
        unsafe {
            mbedtls_ssl_set_bio(
                &mut *ssl_context,
                core::ptr::from_ref(waker_context.as_ref()).cast_mut().cast(),
                Some(send_callback),
                Some(recv_callback),
                None,
            )
        };
        Self {
            context: waker_context,
            ssl_config,
            ssl_context,
            recv_buff: [0u8; MBEDTLS_SSL_IN_CONTENT_LEN as usize],
        }
    }
}

unsafe extern "C" fn recv_callback(context: *mut c_void, buffer: *mut u8, length: usize) -> c_int {
    let context = unsafe { &*context.cast::<RefCell<DtlsWakerContext<'_>>>() };
    let context = context.borrow();
    let Some(waker) = &context.waker else {
        return MBEDTLS_ERR_SSL_INTERNAL_ERROR;
    };
    let mut cx = Context::from_waker(waker);
    let buf = unsafe { core::slice::from_raw_parts_mut(buffer, length) };
    match context.socket.poll_recv(&mut cx, buf) {
        Poll::Ready(Ok(0)) => {
            // 0 length datagrams are not the end of the stream
            waker.wake_by_ref();
            MBEDTLS_ERR_SSL_WANT_READ
        }
        Poll::Ready(Ok(count)) if count <= length => {
            c_int::try_from(count).unwrap_or(MBEDTLS_ERR_SSL_INTERNAL_ERROR)
        }
        Poll::Pending => MBEDTLS_ERR_SSL_WANT_READ,
        Poll::Ready(_) => MBEDTLS_ERR_SSL_INTERNAL_ERROR,
    }
}

unsafe extern "C" fn send_callback(
    context: *mut c_void,
    buffer: *const u8,
    length: usize,
) -> c_int {
    todo!()
}

struct DtlsWakerContext<'net> {
    waker: Option<Waker>,
    socket: Box<dyn UdpSocket + 'net>,
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

/// Reads a single DTLS record.
/// The caller must stop polling after a `DtlsReadError::Tls` error.
impl Stream for UdpDtlsSocket<'_> {
    type Item = Result<(AudioPacketHeader, alloc::boxed::Box<[u8]>), DtlsReadError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        match &mut this.context.borrow_mut().waker {
            Some(waker) => waker.clone_from(cx.waker()),
            slot @ None => *slot = Some(cx.waker().clone()),
        }

        let result = unsafe {
            mbedtls_ssl_read(
                &mut *this.ssl_context,
                this.recv_buff.as_mut_ptr(),
                this.recv_buff.len(),
            )
        };

        match result {
            MBEDTLS_ERR_SSL_WANT_READ | MBEDTLS_ERR_SSL_WANT_WRITE => Poll::Pending,
            0 | MBEDTLS_ERR_SSL_PEER_CLOSE_NOTIFY => Poll::Ready(None),
            code if code < 0 => Poll::Ready(Some(Err(DtlsReadError::Tls { code }))),
            count => {
                let record = &self.recv_buff[..count as usize];
                let [high, low, stream_id, pcm @ ..] = record else {
                    return Poll::Ready(Some(Err(DtlsReadError::TruncatedHeader {
                        actual: record.len(),
                    })));
                };
                let header = AudioPacketHeader {
                    sequence: u16::from_be_bytes([*high, *low]),
                    stream_id: *stream_id,
                };
                Poll::Ready(Some(Ok((header, pcm.into()))))
            }
        }
    }
}
