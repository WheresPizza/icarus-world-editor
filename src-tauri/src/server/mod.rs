pub mod launcher;
pub mod steam;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
}

impl Default for ServerStatus {
    fn default() -> Self {
        ServerStatus::Stopped
    }
}

#[derive(Debug)]
pub struct ServerState {
    pub status: ServerStatus,
    pub pid: Option<u32>,
    pub start_time: Option<std::time::Instant>,
    pub log_lines: Vec<String>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            status: ServerStatus::Stopped,
            pid: None,
            start_time: None,
            log_lines: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub executable_path: Option<String>,
    pub server_name: String,
    pub port: u16,
    pub max_players: u8,
    pub password: Option<String>,
    pub admin_password: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            executable_path: None,
            server_name: "ICARUS Dedicated Server".to_string(),
            port: 17777,
            max_players: 8,
            password: None,
            admin_password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub status: ServerStatus,
    pub pid: Option<u32>,
    pub uptime_secs: Option<u64>,
    pub log_lines: Vec<String>,
}
