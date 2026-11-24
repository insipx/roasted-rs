use alloc::string::String;
use core::ffi::c_int;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStreamError {
    // this is a bit lazy but OK for now
    #[error("{0}")]
    ReadFailed(String),
}

pub type Result<T> = core::result::Result<T, DataStreamError>;

#[derive(Debug, thiserror::Error)]
pub enum DtlsReadError {
    #[error("audio record has {actual} bytes, but its header requires 3")]
    TruncatedHeader { actual: usize },

    #[error("Mbed TLS read failed with status {code}")]
    Tls { code: c_int },
}

#[derive(Debug, thiserror::Error)]
pub enum DtlsSendError {
    #[error("a DTLS record is already queued for writing")]
    WritePending,

    #[error("Mbed TLS write failed with status {code}")]
    Tls { code: c_int },

    #[error("Mbed TLS accepted {written} of {expected} bytes in a DTLS record")]
    ShortWrite { written: usize, expected: usize },
}

#[derive(Debug, thiserror::Error)]
#[error("failed to fill the DTLS random byte buffer")]
pub struct DtlsRngError;

#[derive(Debug, Error)]
#[error("Mbed TLS {operation} failed with status {code}")]
pub struct DtlsSetupError {
    pub operation: &'static str,
    pub code: c_int,
}
