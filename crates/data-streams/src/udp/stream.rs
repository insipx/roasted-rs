use super::*;

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
            MBEDTLS_ERR_SSL_WANT_READ | MBEDTLS_ERR_SSL_WANT_WRITE => {
                this.poll_timer(cx);
                Poll::Pending
            }
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
