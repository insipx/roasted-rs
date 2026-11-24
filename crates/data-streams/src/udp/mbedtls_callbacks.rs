//! Register callback logic with mbedtls_ssl

use mbedtls_rs_sys::MBEDTLS_ERR_ENTROPY_SOURCE_FAILED;

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
    let Ok(sent) = c_int::try_from(length) else {
        return MBEDTLS_ERR_SSL_INTERNAL_ERROR;
    };
    let context = unsafe { &*context.cast::<RefCell<DtlsWakerContext<'_>>>() };
    let context = context.borrow();
    let Some(waker) = &context.waker else {
        return MBEDTLS_ERR_SSL_INTERNAL_ERROR;
    };
    let mut cx = Context::from_waker(waker);
    let buf =
        if length == 0 { &[] } else { unsafe { core::slice::from_raw_parts(buffer, length) } };

    match context.socket.poll_send(&mut cx, buf) {
        Poll::Ready(Ok(count)) if count == length => sent,
        Poll::Pending => MBEDTLS_ERR_SSL_WANT_WRITE,
        Poll::Ready(_) => MBEDTLS_ERR_SSL_INTERNAL_ERROR, // cannot retry partial send with dtls
    }
}

pub(super) unsafe extern "C" fn rng_callback(
    context: *mut c_void,
    output: *mut u8,
    len: usize,
) -> c_int {
    if len == 0 {
        return 0;
    }
    let state = unsafe { &*context.cast::<Rng<'_>>() };
    unsafe { output.write_bytes(0, len) };
    let out = unsafe { core::slice::from_raw_parts_mut(output, len) };
    match state.borrow_mut().fill(out) {
        Ok(()) => 0,
        Err(_) => MBEDTLS_ERR_ENTROPY_SOURCE_FAILED,
    }
}
