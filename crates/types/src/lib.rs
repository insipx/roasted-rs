//! Shared protocol types for roasted.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// Types specific to Daemon communication
#[cfg(any(feature = "daemon", feature = "all"))]
pub mod daemon;

/// WebSocket interface types
#[cfg(any(feature = "ws", feature = "all"))]
pub mod ws;

/// Types for Zippy, the software running on an esp32-s3
#[cfg(any(feature = "zippy", feature = "all"))]
pub mod zippy;

/// SQLite database types enabled with the `diesel` feature
#[cfg(feature = "db")]
pub mod db;
