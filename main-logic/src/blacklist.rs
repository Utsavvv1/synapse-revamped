pub struct Blacklist {
    blocked: Vec<String>,
}

impl Blacklist {
    pub fn new() -> Self {
        Self {
            blocked: vec![
                "chrome.exe".into(),
                "discord.exe".into(),
                "spotify.exe".into(),
                "vlc.exe".into(),
            ],
        }
    }

    pub fn is_blocked(&self, process_name: &str) -> bool {
        self.blocked
            .iter()
            .any(|blocked_name| blocked_name.eq_ignore_ascii_case(process_name))
    }

    pub fn list(&self) -> &[String] {
        &self.blocked
    }
}
