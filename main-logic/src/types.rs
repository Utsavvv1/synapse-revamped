//! Shared newtypes for strong typing across the codebase.

use serde::Serialize;

/// Type-safe wrapper for session IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub i64);

impl From<i64> for SessionId {
    fn from(id: i64) -> Self {
        SessionId(id)
    }
}

impl Into<i64> for SessionId {
    fn into(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Serialize)]
pub struct AppUsageEvent {
    pub process_name: String,
    pub status: String, // "allowed" or "blocked"
    pub session_id: Option<i64>,
    pub start_time: i64,
    pub end_time: i64,
    pub duration_secs: i64,
} 
