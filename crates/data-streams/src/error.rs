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
