mod discovery;
mod files;
pub mod picker;
mod process;
mod protocol;
use protocol::parse_result;

use super::settings::ConverterConfig;
pub use files::ConversionFile;
use files::{inspect, Staging, TARGETS};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant, SystemTime},
};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    pub status: String,
    pub version: Option<String>,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub job_id: String,
    pub running: bool,
    pub files: Vec<ConversionFile>,
    pub target: String,
    pub message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub files: Vec<ConversionFile>,
    pub targets: Vec<String>,
    pub unavailable_targets: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Routes {
    targets: Vec<String>,
    #[serde(default)]
    with_engines: Vec<String>,
}

#[derive(Default)]
struct Cache {
    identity: Option<(PathBuf, Option<SystemTime>)>,
    availability: Availability,
    routes: HashMap<String, Routes>,
}

/// Holds only bounded metadata when idle. Blocking workers exist only during a request.
pub struct ConverterManager {
    gate: Mutex<()>,
    cache: Mutex<Cache>,
    snapshot: Mutex<Snapshot>,
    recent_outputs: Mutex<Vec<String>>,
    cancel: AtomicBool,
    active: AtomicBool,
}

impl Default for ConverterManager {
    fn default() -> Self {
        Self::new()
    }
}
impl ConverterManager {
    pub fn new() -> Self {
        Self {
            gate: Mutex::new(()),
            cache: Mutex::new(Cache::default()),
            snapshot: Mutex::new(Snapshot::default()),
            recent_outputs: Mutex::new(Vec::new()),
            cancel: AtomicBool::new(false),
            active: AtomicBool::new(false),
        }
    }

    fn executable(&self, config: &ConverterConfig) -> Result<PathBuf, String> {
        if !config.enabled {
            return Err("Enable file conversion in Settings → Tools.".into());
        }
        discovery::executable(config.cli_path.as_deref())
            .ok_or_else(|| "Filee CLI was not found. Install Filee or locate filee-cli.exe.".into())
    }

    fn probe(&self, executable: &Path, refresh: bool) -> Result<Availability, String> {
        let identity = (
            executable.to_path_buf(),
            std::fs::metadata(executable)
                .ok()
                .and_then(|m| m.modified().ok()),
        );
        {
            let cache = self.cache.lock().map_err(|e| e.to_string())?;
            if !refresh
                && cache.identity.as_ref() == Some(&identity)
                && cache.availability.status == "ready"
            {
                return Ok(cache.availability.clone());
            }
        }
        let never_cancel = AtomicBool::new(false);
        let version = process::run(
            executable,
            &["version".into()],
            Duration::from_secs(15),
            &never_cancel,
            |_| {},
        )?;
        if version.code != 0 {
            return Err(
                "Filee could not report its CLI version. Update Filee and try again.".into(),
            );
        }
        let version = String::from_utf8_lossy(&version.stdout).trim().to_string();
        if !version.to_lowercase().starts_with("filee ") || version.len() > 128 {
            return Err(
                "This executable does not identify itself as a supported Filee CLI.".into(),
            );
        }
        let formats = process::run(
            executable,
            &["formats".into(), "--json".into()],
            Duration::from_secs(15),
            &never_cancel,
            |_| {},
        )?;
        let json: serde_json::Value = serde_json::from_slice(&formats.stdout).map_err(|_| {
            "Update Filee: its CLI does not support the expected JSON format list.".to_string()
        })?;
        if formats.code != 0 || !json["formats"].is_array() {
            return Err("Update Filee to a version with JSON CLI support.".into());
        }
        let availability = Availability {
            status: "ready".into(),
            version: Some(version),
            message: "Ready · offline conversion".into(),
        };
        *self.cache.lock().map_err(|e| e.to_string())? = Cache {
            identity: Some(identity),
            availability: availability.clone(),
            routes: HashMap::new(),
        };
        Ok(availability)
    }

    pub fn status(&self, config: &ConverterConfig, refresh: bool) -> Availability {
        let executable = match self.executable(config) {
            Ok(path) => path,
            Err(message) => {
                return Availability {
                    status: "missing".into(),
                    message,
                    version: None,
                }
            }
        };
        let Ok(_gate) = self.gate.try_lock() else {
            return self
                .cache
                .lock()
                .map(|c| c.availability.clone())
                .unwrap_or_default();
        };
        self.probe(&executable, refresh)
            .unwrap_or_else(|message| Availability {
                status: "incompatible".into(),
                message,
                version: None,
            })
    }

    fn routes(
        &self,
        executable: &Path,
        extension: &str,
        cancel: &AtomicBool,
    ) -> Result<Routes, String> {
        if let Some(routes) = self
            .cache
            .lock()
            .map_err(|e| e.to_string())?
            .routes
            .get(extension)
            .cloned()
        {
            return Ok(routes);
        }
        let result = process::run(
            executable,
            &["formats".into(), extension.into(), "--json".into()],
            Duration::from_secs(15),
            cancel,
            |_| {},
        )?;
        if result.code != 0 {
            return Err(format!("Filee could not inspect .{extension} conversions. Update Filee and check its engines."));
        }
        let routes: Routes = serde_json::from_slice(&result.stdout).map_err(|_| {
            "Filee returned an incompatible format response. Update Filee.".to_string()
        })?;
        let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
        if cache.routes.len() >= 32 {
            cache.routes.clear();
        }
        cache.routes.insert(extension.into(), routes.clone());
        Ok(routes)
    }

    pub fn inspect(
        &self,
        config: &ConverterConfig,
        paths: Vec<String>,
    ) -> Result<Inspection, String> {
        let _gate = self
            .gate
            .try_lock()
            .map_err(|_| "Conversion is busy. Wait for the current operation.".to_string())?;
        let executable = self.executable(config)?;
        let files = inspect(paths)?;
        self.probe(&executable, false)?;
        let never_cancel = AtomicBool::new(false);
        self.inspect_routes(&executable, files, &never_cancel)
    }

    fn inspect_routes(
        &self,
        executable: &Path,
        files: Vec<ConversionFile>,
        cancel: &AtomicBool,
    ) -> Result<Inspection, String> {
        let mut available: HashSet<String> = TARGETS.iter().map(|s| s.to_string()).collect();
        let mut potential = available.clone();
        for file in &files {
            let routes = self.routes(executable, &file.extension, cancel)?;
            available.retain(|target| routes.targets.contains(target));
            potential.retain(|target| {
                routes.targets.contains(target) || routes.with_engines.contains(target)
            });
        }
        let targets = TARGETS
            .iter()
            .filter(|target| available.contains(**target))
            .map(|s| s.to_string())
            .collect();
        let unavailable_targets = TARGETS
            .iter()
            .filter(|target| potential.contains(**target) && !available.contains(**target))
            .map(|s| s.to_string())
            .collect();
        Ok(Inspection {
            files,
            targets,
            unavailable_targets,
        })
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn result_path(&self, path: &str) -> Result<PathBuf, String> {
        let current = self.snapshot();
        let recent = self.recent_outputs.lock().map_err(|e| e.to_string())?;
        if !recent.iter().any(|output| output == path)
            && !current
                .files
                .iter()
                .any(|file| file.outputs.iter().any(|output| output == path))
        {
            return Err("Only completed conversion results can be opened.".into());
        }
        let path = std::fs::canonicalize(path)
            .map_err(|_| "The result was moved or deleted.".to_string())?;
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !path.is_file() || (!TARGETS.contains(&extension.as_str()) && extension != "jpeg") {
            return Err("This result type cannot be opened from the converter.".into());
        }
        Ok(path)
    }

    pub fn open_result(&self, path: &str, folder: bool) -> Result<(), String> {
        let path = self.result_path(path)?;
        if folder {
            tauri_plugin_opener::reveal_item_in_dir(path).map_err(|e| e.to_string())
        } else {
            tauri_plugin_opener::open_path(path, None::<&str>)
                .map_err(|e| e.to_string())
        }
    }

    fn emit(&self, app: &AppHandle) {
        let _ = app.emit("converter.changed", self.snapshot());
    }

    pub fn cancel(&self, job_id: &str) -> Result<(), String> {
        let snapshot = self.snapshot();
        if snapshot.running && snapshot.job_id == job_id {
            self.cancel.store(true, Ordering::Release);
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        self.cancel.store(true, Ordering::Release);
        let start = Instant::now();
        while self.active.load(Ordering::Acquire) && start.elapsed() < Duration::from_secs(3) {
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn convert(
        &self,
        app: &AppHandle,
        config: &ConverterConfig,
        paths: Vec<String>,
        target: String,
        output_folder: Option<String>,
    ) -> Result<Snapshot, String> {
        let _gate = self
            .gate
            .try_lock()
            .map_err(|_| "Conversion is busy. Wait for the current operation.".to_string())?;
        if !TARGETS.contains(&target.as_str()) {
            return Err("Choose a supported target format.".into());
        }
        let executable = self.executable(config)?;
        let files = inspect(paths)?;
        self.cancel.store(false, Ordering::Release);
        self.probe(&executable, false)?;
        let inspection = self.inspect_routes(&executable, files, &self.cancel)?;
        if !inspection.targets.contains(&target) {
            return Err("That format is unavailable for this selection. Check Filee's engines or remove incompatible files.".into());
        }
        let folder = output_folder.map(PathBuf::from);
        if let Some(path) = &folder {
            if !path.is_absolute() || !path.is_dir() {
                return Err("Choose an existing output folder.".into());
            }
        }
        *self.snapshot.lock().map_err(|e| e.to_string())? = Snapshot {
            job_id: files::identity(),
            running: true,
            files: inspection.files,
            target: target.clone(),
            message: "Converting…".into(),
        };
        self.active.store(true, Ordering::Release);
        self.emit(app);
        let count = self.snapshot().files.len();
        for index in 0..count {
            if self.cancel.load(Ordering::Acquire) {
                break;
            }
            self.snapshot.lock().unwrap().files[index].state = "running".into();
            self.emit(app);
            let source = self.snapshot().files[index].path.clone();
            let parent = folder.clone().unwrap_or_else(|| {
                Path::new(&source)
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join("Converted")
            });
            let result = self.convert_file(app, &executable, &source, &target, &parent, index);
            let mut snapshot = self.snapshot.lock().unwrap();
            let file = &mut snapshot.files[index];
            match result {
                Ok(outputs) => {
                    file.state = "done".into();
                    file.progress = Some(1.0);
                    // The drawer opens the first result; Folder exposes all pages/assets.
                    // Retain one path per file instead of keeping thousands of PDF page paths.
                    file.outputs = outputs.into_iter().take(1).collect();
                    if let Some(output) = file.outputs.first() {
                        let mut recent = self.recent_outputs.lock().unwrap();
                        recent.insert(0, output.clone());
                        recent.truncate(10);
                    }
                }
                Err(error) => {
                    file.state = if self.cancel.load(Ordering::Acquire) {
                        "cancelled"
                    } else {
                        "failed"
                    }
                    .into();
                    file.error = Some(error);
                }
            }
            drop(snapshot);
            self.emit(app);
        }
        let mut snapshot = self.snapshot.lock().unwrap();
        for file in &mut snapshot.files {
            if file.state == "pending" {
                file.state = "cancelled".into();
            }
        }
        snapshot.running = false;
        let done = snapshot.files.iter().filter(|f| f.state == "done").count();
        snapshot.message = if self.cancel.load(Ordering::Acquire) {
            format!("Cancelled · {done} converted")
        } else {
            format!("{done} of {count} converted")
        };
        let result = snapshot.clone();
        drop(snapshot);
        self.active.store(false, Ordering::Release);
        self.emit(app);
        Ok(result)
    }

    fn convert_file(
        &self,
        app: &AppHandle,
        executable: &Path,
        source: &str,
        target: &str,
        parent: &Path,
        index: usize,
    ) -> Result<Vec<String>, String> {
        let stage = Staging::new(parent)?;
        let arguments = vec![
            "convert".into(),
            source.into(),
            "--to".into(),
            target.into(),
            "--json".into(),
            "--skip".into(),
            "-o".into(),
            stage.path.to_string_lossy().into_owned(),
        ];
        let mut last = Instant::now() - Duration::from_secs(1);
        let result = process::run(
            executable,
            &arguments,
            Duration::from_secs(300),
            &self.cancel,
            |fraction| {
                if last.elapsed() >= Duration::from_millis(150) {
                    self.snapshot.lock().unwrap().files[index].progress = Some(fraction);
                    self.emit(app);
                    last = Instant::now();
                }
            },
        )?;
        if self.cancel.load(Ordering::Acquire) {
            return Err("Conversion cancelled.".into());
        }
        let outputs = parse_result(&result.stdout, result.code).map_err(|error| {
            // Keep engine diagnostics bounded in memory, but don't expose an unbounded log in UI.
            if result.diagnostics.contains("OutOfMemory") {
                "This file exceeded the converter memory budget. Try a smaller file.".into()
            } else {
                error
            }
        })?;
        stage.publish(source, target, &outputs)
    }
}

#[cfg(test)]
mod result_access_tests {
    use super::*;
    #[test]
    fn result_opening_requires_an_owned_data_file() {
        let root = std::env::temp_dir().join(format!("notch-result-{}", files::identity()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("photo.jpg");
        std::fs::write(&path, b"result").unwrap();
        let text = path.to_string_lossy().into_owned();
        let manager = ConverterManager::new();
        assert!(manager.result_path(&text).is_err());
        manager.recent_outputs.lock().unwrap().push(text.clone());
        assert!(manager.result_path(&text).is_ok());
        let executable = root.join("program.exe");
        std::fs::write(&executable, b"not a data format").unwrap();
        let executable_text = executable.to_string_lossy().into_owned();
        manager.recent_outputs.lock().unwrap().push(executable_text.clone());
        assert!(manager.result_path(&executable_text).is_err());
        std::fs::remove_file(path).unwrap();
        assert!(manager.result_path(&text).is_err());
        std::fs::remove_file(executable).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
