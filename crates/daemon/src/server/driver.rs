//! Drives the execution of the Roasted event loop.
//!  - Starts the Gaggimate Listener
//!  - Asks for audio feedback from ESP32/begins conversation sequences
//!  - Handles communication with Ollama local LLM
//!  - Spawned by server upon connection with Zippy

use crate::listeners::gaggimate::GaggimateStream;

pub struct RoastedDriver {}
