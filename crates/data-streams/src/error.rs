use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStreamError {
    // this is a bit lazy but OK for now
    #[error("{0}")]
    ReadFailed(String),
}

pub type Result<T> = core::result::Result<T, DataStreamError>;
