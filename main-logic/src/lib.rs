//! # Synapse Logic Crate
//!
//! This crate contains the core logic for the Synapse application,
//! including session management, application rule handling, database interaction,
//! and platform-specific utilities.

// Make modules public so users can access sub-items if needed.
pub mod apprules;
pub mod db;
pub mod error;
pub mod graceful_shutdown;
pub mod logger;
pub mod metrics;
pub mod platform;
pub mod session;
pub mod types;
pub mod constants;
pub mod sync;

// Re-export key types for a cleaner public API.
pub use apprules::AppRules;
pub use db::DbHandle;
pub use error::SynapseError;
pub use metrics::Metrics;
pub use session::{FocusSession, SessionManager};
pub use types::SessionId; 
