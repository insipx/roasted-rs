//! Shared protocol types for roasted.

#![deny(missing_docs)]

pub mod daemon;
pub mod ws;
/// Types for Zippy, the software running on an esp32-s3
pub mod zippy;

/// SQLite database types enabled with the `diesel` feature
#[cfg(feature = "diesel")]
pub mod db;
