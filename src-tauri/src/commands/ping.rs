use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PingResponse {
    pub message: String,
    pub version: String,
    pub timestamp_ms: u64,
    pub backend_ready: bool,
}

#[tauri::command]
pub fn ping(client_timestamp_ms: Option<u64>) -> PingResponse {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let _ = client_timestamp_ms;

    PingResponse {
        message: "pong from Project Notch backend".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp_ms: now,
        backend_ready: true,
    }
}
