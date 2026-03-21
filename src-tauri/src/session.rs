//! session.rs — Re-exports for backwards compatibility.
//!
//! The session module has been split into:
//! - `session_types.rs` — structs, enums, DTOs, constants
//! - `session_manager.rs` — SessionManager struct + core methods
//! - `session_reading.rs` — on_reading() state machine logic

pub use crate::session_manager::*;
pub use crate::session_types::*;
