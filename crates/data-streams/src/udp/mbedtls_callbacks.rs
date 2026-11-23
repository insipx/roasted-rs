//! Register callback logic with mbedtls_ssl

use super::*;

pub(super) unsafe extern "C" fn recv_callback(
    context: *mut c_void,
    buffer: *mut u8,
    length: usize,
) -> c_int {
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

pub(super) unsafe extern "C" fn send_callback(
    context: *mut c_void,
    buffer: *const u8,
    length: usize,
) -> c_int {
    todo!()
}

pub(super) unsafe extern "C" fn timer_callback() -> c_int {
    todo!()
}
