use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    pub is_ready: Mutex<bool>,
}
