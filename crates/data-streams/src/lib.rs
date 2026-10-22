#![cfg_attr(not(feature = "std"), no_std)]
#![feature(core_io)]

mod cobs_stream;
mod error;
mod udp;
pub mod zippy;

extern crate alloc;
