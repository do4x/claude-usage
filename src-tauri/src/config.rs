use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub session_cookie: String,
    pub poll_interval_secs: u64,
    pub org_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            session_cookie: String::new(),
            poll_interval_secs: 120,
            org_id: String::new(),
        }
    }
}
