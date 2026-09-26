//! Read-only GaggiMate shot telemetry on `/ws`.
//!
//! Status fields follow upstream revision `001a475105cdfa11c54e2c688046f8159dda8d3e`:
//! <https://github.com/jniebuhr/gaggimate/blob/001a475105cdfa11c54e2c688046f8159dda8d3e/docs/websocket-api.yaml>.
//! Process telemetry supplements that schema from upstream
//! `src/display/plugins/WebSocketHandler.cpp` (`publishTelemetry`).
//!
//!
//! ```
//! use roasted_types::ws::gaggimate::{Message, Patch};
//! let message: Message = serde_json::from_str(r#"{"tp":"evt:status","pr":8.5}"#)?;
//! if let Message::Status(status) = message {
//!     assert_eq!(status.current_pressure, Patch::Value(8.5));
//!     assert_eq!(status.current_temperature, Patch::Absent);
//! }
//! # Ok::<(), serde_json::Error>(())
//! ```

mod elapsed_ms;
mod other;
mod patch;
mod process_status;
mod status;
mod utility_flag;

pub use elapsed_ms::*;
pub use other::*;
pub use patch::*;
pub use process_status::*;
pub use status::*;
pub use utility_flag::*;
