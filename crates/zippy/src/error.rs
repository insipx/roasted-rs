use esp_hal::uart::ConfigError;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to start UART Writer {0}")]
    UartConfig(#[from] ConfigError),
    #[error("Error deserializing CLI commands over USB JTag {0}")]
    Serialization(#[from] postcard::Error),
    #[error("{0}")]
    Capacity(#[from] heapless::CapacityError),
}
