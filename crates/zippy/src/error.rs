use esp_hal::uart::ConfigError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to start UART Writer {0}")]
    UartConfig(#[from] ConfigError),
    #[error("Error deserializing CLI commands over USB JTag {0}")]
    Serialization(#[from] postcard::Error),
    #[error("{0}")]
    Capacity(#[from] heapless::CapacityError),
    #[error("Error reading/writing data to flash storage {0}")]
    Storage(#[from] StorageError),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("{0}")]
    Partition(#[from] esp_bootloader_esp_idf::partitions::Error),
    #[error("{0}")]
    Postcard(#[from] postcard::Error),
    #[error("error generating entropy for encrypted storage")]
    Rng,
    #[error("Error encrypting data for persistent storage")]
    Encryption(#[from] chacha20poly1305::Error),
}

impl From<esp_hal::rng::TrngError> for StorageError {
    fn from(_: esp_hal::rng::TrngError) -> Self {
        StorageError::Rng
    }
}
