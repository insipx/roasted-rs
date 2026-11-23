use futures_util::sink::Sink;
use mbedtls_rs_sys::{mbedtls_ssl_close_notify, mbedtls_ssl_write};

use super::*;
use crate::error::DtlsSendError;

impl Sink<AudioPacket> for UdpDtlsSocket<'_> {
    type Error = DtlsSendError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.pending_send.is_none() {
            return Poll::Ready(Ok(()));
        }

        self.poll_flush(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: AudioPacket) -> Result<(), Self::Error> {
        let this = self.get_mut();
        if this.pending_send.is_some() {
            return Err(DtlsSendError::WritePending);
        }

        let mut record = Vec::with_capacity(3 + item.pcm.len());
        record.extend_from_slice(&item.header.sequence.to_be_bytes());
        record.push(item.header.stream_id);
        record.extend_from_slice(item.pcm);
        this.pending_send = Some(record);
        Ok(())
    }

    // A write error is terminal for this DTLS session.
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.as_mut().get_mut();
        let Some(send_buf) = this.pending_send.as_ref() else {
            return Poll::Ready(Ok(()));
        };
        match &mut this.context.borrow_mut().waker {
            Some(waker) => waker.clone_from(cx.waker()),
            slot @ None => *slot = Some(cx.waker().clone()),
        }
        let result =
            unsafe { mbedtls_ssl_write(&mut *this.ssl_context, send_buf.as_ptr(), send_buf.len()) };

        match result {
            MBEDTLS_ERR_SSL_WANT_READ | MBEDTLS_ERR_SSL_WANT_WRITE => {
                this.poll_timer(cx);
                Poll::Pending
            }
            code if code < 0 => Poll::Ready(Err(DtlsSendError::Tls { code })),
            written if written as usize == send_buf.len() => {
                this.pending_send = None;
                Poll::Ready(Ok(()))
            }
            written => Poll::Ready(Err(DtlsSendError::ShortWrite {
                written: written as usize,
                expected: send_buf.len(),
            })),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let _ = self.as_mut().poll_flush(cx)?;
        let this = self.as_mut().get_mut();
        match &mut this.context.borrow_mut().waker {
            Some(waker) => waker.clone_from(cx.waker()),
            slot @ None => *slot = Some(cx.waker().clone()),
        }

        let result = unsafe { mbedtls_ssl_close_notify(&mut *this.ssl_context) };

        match result {
            MBEDTLS_ERR_SSL_WANT_READ | MBEDTLS_ERR_SSL_WANT_WRITE => Poll::Pending,
            code if code < 0 => Poll::Ready(Err(DtlsSendError::Tls { code })),
            // TODO
            // just call anything 0 or more success
            // in reality if something else is returned stop using connection
            // this is not handled yet.
            // https://docs.rs/mbedtls-rs-sys/latest/mbedtls_rs_sys/fn.mbedtls_ssl_close_notify.html
            success if success >= 0 => Poll::Ready(Ok(())),
            _ => unreachable!(),
        }
    }
}
