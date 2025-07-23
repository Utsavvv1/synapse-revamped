//! Shared newtypes for strong typing across the codebase.

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
