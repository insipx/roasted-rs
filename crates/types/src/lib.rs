//! Shared protocol types for roasted.

#![deny(missing_docs)]

pub mod ws;

pub mod daemon;

/// SQLite database types enabled with the `diesel` feature
#[cfg(feature = "diesel")]
pub mod db;
