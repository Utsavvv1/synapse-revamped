use std::time::{SystemTime, Duration};
use std::fs::OpenOptions;
use std::io::Write;
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct FocusSession {
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub work_apps: Vec<String>,
    pub is_active: bool,
}

impl FocusSession {
    pub fn new(work_apps: Vec<String>) -> Self {
        Self {
            start_time: SystemTime::now(),
            end_time: None,
            work_apps,
            is_active: true,
        }
    }

    pub fn end(&mut self) {
        self.end_time = Some(SystemTime::now());
        self.is_active = false;
    }

    pub fn duration(&self) -> Option<Duration> {
        self.end_time
            .and_then(|end| end.duration_since(self.start_time).ok())
    }

    pub fn log_to_file(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("synapse.log")
            {
                let _ = writeln!(file, "SESSION: {}", json);
            }
        }
    }
}
