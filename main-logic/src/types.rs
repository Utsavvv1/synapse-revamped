//! Shared newtypes for strong typing across the codebase.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionId(pub Uuid);

impl From<Uuid> for SessionId {
    fn from(id: Uuid) -> Self {
        SessionId(id)
    }
}

impl Into<Uuid> for SessionId {
    fn into(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppUsageEvent {
    pub id: Uuid,
    pub process_name: String,
    pub status: String, // "allowed" or "blocked"
    pub session_id: Option<Uuid>,
    pub start_time: i64,
    pub end_time: i64,
    pub duration_secs: i64,
}
