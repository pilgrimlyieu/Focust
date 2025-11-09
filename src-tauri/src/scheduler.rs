pub mod event;
pub mod manager;
pub mod models;
pub mod shared_state;

mod attention_timer;
mod break_scheduler;
mod event_emitter;

// Re-export public API
pub use models::*;
pub use shared_state::{SharedSchedulerState, SharedState, create_shared_state};
