use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

use crate::integrations::startup;
use crate::window;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub launch_at_startup: bool,
    pub selected_monitor: Option<String>,
    pub always_on_top: bool,
    pub auto_collapse_delay_ms: u32,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            selected_monitor: None,
            always_on_top: true,
            auto_collapse_delay_ms: 4000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceConfig {
    pub accent_color: String,
    pub opacity: f32,
    pub animation_intensity: f32,
    pub reduced_motion: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            accent_color: "#3b82f6".to_string(),
            opacity: 0.96,
            animation_intensity: 1.0,
            reduced_motion: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MascotConfig {
    pub enabled: bool,
    pub cursor_tracking: bool,
    pub animation_intensity: f32,
    pub sound_enabled: bool,
}

impl Default for MascotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cursor_tracking: true,
            animation_intensity: 1.0,
            sound_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MusicConfig {
    pub enabled: bool,
    pub show_controls: bool,
}

impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_controls: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimerConfig {
    pub default_preset: u32,
    pub custom_minutes: u32,
    pub auto_start_on_preset_select: bool,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            default_preset: 25,
            custom_minutes: 15,
            auto_start_on_preset_select: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardConfig {
    pub enabled: bool,
    pub history_limit: u32,
    pub persist_history: bool,
    pub clear_on_exit: bool,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_limit: 25,
            persist_history: false, // Local-first privacy: OFF by default
            clear_on_exit: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DownloadsConfig {
    pub enabled: bool,
    pub watched_folders: Vec<String>,
    pub auto_open_completed: bool,
}

impl Default for DownloadsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            watched_folders: Vec::new(),
            auto_open_completed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppsConfig {
    pub quick_dock_enabled: bool,
    pub max_quick_pinned: usize,
}

impl Default for AppsConfig {
    fn default() -> Self {
        Self {
            quick_dock_enabled: true,
            max_quick_pinned: 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceConfig {
    pub battery_saver_mode: bool,
    pub pause_inactive_animations: bool,
    pub target_fps: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            battery_saver_mode: false,
            pause_inactive_animations: true,
            target_fps: 60,
        }
    }
}

/// Unified typed settings configuration.
/// Defined in PRODUCT_SPEC.md section 22.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsConfig {
    pub schema_version: u32,
    pub general: GeneralConfig,
    pub appearance: AppearanceConfig,
    pub mascot: MascotConfig,
    pub music: MusicConfig,
    pub timer: TimerConfig,
    pub clipboard: ClipboardConfig,
    pub downloads: DownloadsConfig,
    pub apps: AppsConfig,
    pub performance: PerformanceConfig,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            mascot: MascotConfig::default(),
            music: MusicConfig::default(),
            timer: TimerConfig::default(),
            clipboard: ClipboardConfig::default(),
            downloads: DownloadsConfig::default(),
            apps: AppsConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Migrates and sanitizes raw JSON into a valid current-version SettingsConfig.
/// Recovers safely from missing sections, older schemas, or corrupted types.
pub fn migrate_settings_json(raw: serde_json::Value) -> SettingsConfig {
    let mut config = SettingsConfig::default();

    if let serde_json::Value::Object(map) = raw {
        let _version = map
            .get("schemaVersion")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        // General section migration / recovery
        if let Some(val) = map.get("general") {
            if let Ok(gen) = serde_json::from_value::<GeneralConfig>(val.clone()) {
                config.general = gen;
            } else if let Some(obj) = val.as_object() {
                if let Some(b) = obj.get("launchAtStartup").and_then(|v| v.as_bool()) {
                    config.general.launch_at_startup = b;
                }
                if let Some(s) = obj.get("selectedMonitor").and_then(|v| v.as_str()) {
                    config.general.selected_monitor = Some(s.to_string());
                }
                if let Some(b) = obj.get("alwaysOnTop").and_then(|v| v.as_bool()) {
                    config.general.always_on_top = b;
                }
                if let Some(n) = obj.get("autoCollapseDelayMs").and_then(|v| v.as_u64()) {
                    config.general.auto_collapse_delay_ms = n as u32;
                }
            }
        }

        // Appearance section migration / recovery
        if let Some(val) = map.get("appearance") {
            if let Ok(app) = serde_json::from_value::<AppearanceConfig>(val.clone()) {
                config.appearance = app;
            } else if let Some(obj) = val.as_object() {
                if let Some(c) = obj.get("accentColor").and_then(|v| v.as_str()) {
                    config.appearance.accent_color = c.to_string();
                }
                if let Some(o) = obj.get("opacity").and_then(|v| v.as_f64()) {
                    config.appearance.opacity = o as f32;
                }
                if let Some(i) = obj.get("animationIntensity").and_then(|v| v.as_f64()) {
                    config.appearance.animation_intensity = i as f32;
                }
                if let Some(b) = obj.get("reducedMotion").and_then(|v| v.as_bool()) {
                    config.appearance.reduced_motion = b;
                }
            }
        }

        // Mascot section migration / recovery
        if let Some(val) = map.get("mascot") {
            if let Ok(m) = serde_json::from_value::<MascotConfig>(val.clone()) {
                config.mascot = m;
            } else if let Some(obj) = val.as_object() {
                if let Some(b) = obj.get("enabled").and_then(|v| v.as_bool()) {
                    config.mascot.enabled = b;
                }
                if let Some(b) = obj.get("cursorTracking").and_then(|v| v.as_bool()) {
                    config.mascot.cursor_tracking = b;
                }
                if let Some(i) = obj.get("animationIntensity").and_then(|v| v.as_f64()) {
                    config.mascot.animation_intensity = i as f32;
                }
                if let Some(b) = obj.get("soundEnabled").and_then(|v| v.as_bool()) {
                    config.mascot.sound_enabled = b;
                }
            }
        }

        // Music section migration / recovery
        if let Some(val) = map.get("music") {
            if let Ok(m) = serde_json::from_value::<MusicConfig>(val.clone()) {
                config.music = m;
            }
        }

        // Timer section migration / recovery
        if let Some(val) = map.get("timer") {
            if let Ok(t) = serde_json::from_value::<TimerConfig>(val.clone()) {
                config.timer = t;
            }
        }

        // Clipboard section migration / recovery
        if let Some(val) = map.get("clipboard") {
            if let Ok(c) = serde_json::from_value::<ClipboardConfig>(val.clone()) {
                config.clipboard = c;
            }
        }

        // Downloads section migration / recovery
        if let Some(val) = map.get("downloads") {
            if let Ok(d) = serde_json::from_value::<DownloadsConfig>(val.clone()) {
                config.downloads = d;
            }
        }

        // Apps section migration / recovery
        if let Some(val) = map.get("apps") {
            if let Ok(a) = serde_json::from_value::<AppsConfig>(val.clone()) {
                config.apps = a;
            }
        }

        // Performance section migration / recovery
        if let Some(val) = map.get("performance") {
            if let Ok(p) = serde_json::from_value::<PerformanceConfig>(val.clone()) {
                config.performance = p;
            }
        }

        // If migrating from v0 or earlier, ensure current schema version
        config.schema_version = CURRENT_SCHEMA_VERSION;
    }

    config
}

pub struct SettingsManager {
    config: Mutex<SettingsConfig>,
    storage_path: Mutex<Option<PathBuf>>,
    app_handle: Mutex<Option<AppHandle>>,
}

impl SettingsManager {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(SettingsConfig::default()),
            storage_path: Mutex::new(None),
            app_handle: Mutex::new(None),
        }
    }

    /// Initializes settings from the user app data directory.
    /// Handles missing directory, safe fallback on corrupted file, and initial synchronization.
    pub fn init(&self, app_data_dir: PathBuf, app_handle: AppHandle) {
        let _ = fs::create_dir_all(&app_data_dir);
        let path = app_data_dir.join("settings.json");
        *self.storage_path.lock().unwrap() = Some(path.clone());
        *self.app_handle.lock().unwrap() = Some(app_handle.clone());

        let mut loaded_config = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(raw_json) => migrate_settings_json(raw_json),
                    Err(err) => {
                        eprintln!("[SettingsManager] Corrupted settings JSON, recovering defaults: {err}");
                        // Backup corrupted file for diagnostics
                        let backup = app_data_dir.join("settings.corrupted.bak");
                        let _ = fs::copy(&path, backup);
                        SettingsConfig::default()
                    }
                },
                Err(err) => {
                    eprintln!("[SettingsManager] Failed to read settings file: {err}");
                    SettingsConfig::default()
                }
            }
        } else {
            SettingsConfig::default()
        };

        // Query actual Windows autostart status to ensure sync
        loaded_config.general.launch_at_startup = startup::is_autostart_enabled();

        *self.config.lock().unwrap() = loaded_config.clone();

        // Save normalized config to disk
        let _ = self.save_to_disk(&loaded_config);
    }

    pub fn get_settings(&self) -> SettingsConfig {
        let mut cfg = self.config.lock().unwrap().clone();
        cfg.general.launch_at_startup = startup::is_autostart_enabled();
        cfg
    }

    pub fn save_settings(&self, new_config: SettingsConfig) -> Result<SettingsConfig, String> {
        let mut current = self.config.lock().unwrap();

        // 1. Apply Windows autostart registry state if changed
        if current.general.launch_at_startup != new_config.general.launch_at_startup {
            let _ = startup::set_autostart_enabled(new_config.general.launch_at_startup)?;
        }

        // 2. Apply window properties if changed
        if let Some(app) = self.app_handle.lock().unwrap().as_ref() {
            if let Some(webview_window) = app.get_webview_window("main") {
                let win = webview_window.as_ref().window();

                // Always on top
                if current.general.always_on_top != new_config.general.always_on_top {
                    let _ = win.set_always_on_top(new_config.general.always_on_top);
                }

                // Selected monitor repositioning
                if current.general.selected_monitor != new_config.general.selected_monitor {
                    let state_str = "compact"; // Reposition in current state
                    let _ = window::apply_notch_state(
                        &win,
                        state_str,
                        new_config.general.selected_monitor.as_deref(),
                    );
                }

                // Emit event to notify frontend of applied settings
                let _ = webview_window.emit("settings-changed", new_config.clone());
            }
        }

        *current = new_config.clone();
        self.save_to_disk(&new_config)?;

        Ok(new_config)
    }

    pub fn reset_to_defaults(&self) -> Result<SettingsConfig, String> {
        let defaults = SettingsConfig::default();
        self.save_settings(defaults)
    }

    fn save_to_disk(&self, config: &SettingsConfig) -> Result<(), String> {
        let path_guard = self.storage_path.lock().unwrap();
        if let Some(ref path) = *path_guard {
            let json = serde_json::to_string_pretty(config)
                .map_err(|e| format!("Failed to serialize settings: {e}"))?;
            fs::write(path, json)
                .map_err(|e| format!("Failed to write settings to disk: {e}"))?;
        }
        Ok(())
    }
}
