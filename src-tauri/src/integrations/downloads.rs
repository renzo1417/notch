use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DownloadState {
    pub file_name: String,
    pub progress_percent: Option<u8>,
    pub is_completed: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatchedFolder {
    pub id: String,
    pub path: String,
    pub is_default: bool,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct DownloadsConfig {
    pub watched_folders: Vec<WatchedFolder>,
}

/// Recognized partial download file extensions.
pub const KNOWN_PARTIAL_EXTENSIONS: &[&str] = &[
    ".crdownload", // Chromium, Chrome, Edge, Brave, Opera, Vivaldi
    ".part",       // Firefox, Waterfox, LibreWolf
    ".download",   // Safari, various native download managers
    ".!ut",        // uTorrent, BitTorrent
    ".mega",       // Mega client
    ".tmp",        // Temporary downloads
    ".temp",       // Temporary swap
];

/// Extracts the intended destination file name from a recognized partial download file name.
/// Returns None if the file does not have a recognized partial download suffix.
pub fn extract_partial_target_name(file_name: &str) -> Option<String> {
    let lower = file_name.to_lowercase();
    for &ext in KNOWN_PARTIAL_EXTENSIONS {
        if lower.ends_with(ext) && file_name.len() > ext.len() {
            let base = &file_name[..file_name.len() - ext.len()];
            if !base.is_empty() {
                return Some(base.to_string());
            }
        }
    }
    None
}

/// Internal tracking representation of an ongoing download.
#[derive(Debug, Clone)]
struct ActiveDownloadEntry {
    target_name: String,
    target_path: PathBuf,
    last_emitted_ms: u64,
}

/// Internal tracking for a single folder filesystem watcher thread.
struct FolderWatcherHandle {
    stop_signal: Arc<AtomicBool>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    #[cfg(target_os = "windows")]
    raw_dir_handle: isize,
}

impl FolderWatcherHandle {
    pub fn stop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);

        #[cfg(target_os = "windows")]
        if self.raw_dir_handle != 0 {
            use windows::Win32::Foundation::{CloseHandle, HANDLE};
            use windows::Win32::System::IO::CancelIoEx;

            unsafe {
                let h = HANDLE(self.raw_dir_handle as *mut _);
                let _ = CancelIoEx(h, None);
                let _ = CloseHandle(h);
            }
            self.raw_dir_handle = 0;
        }

        if let Some(th) = self.thread_handle.take() {
            let _ = th.join();
        }
    }
}

/// Central state manager for filesystem download awareness and watched folder tracking.
pub struct DownloadsManager {
    app_handle: Mutex<Option<AppHandle>>,
    storage_path: Mutex<Option<PathBuf>>,
    watched_folders: Mutex<Vec<WatchedFolder>>,
    watchers: Mutex<HashMap<String, FolderWatcherHandle>>,
    active_downloads: Arc<Mutex<HashMap<String, ActiveDownloadEntry>>>,
    recently_completed: Arc<Mutex<HashMap<String, u64>>>,
    current_state: Mutex<Option<DownloadState>>,
}

impl DownloadsManager {
    pub fn new() -> Self {
        Self {
            app_handle: Mutex::new(None),
            storage_path: Mutex::new(None),
            watched_folders: Mutex::new(Vec::new()),
            watchers: Mutex::new(HashMap::new()),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            recently_completed: Arc::new(Mutex::new(HashMap::new())),
            current_state: Mutex::new(None),
        }
    }

    /// Initializes storage path, loads configured folders, and starts filesystem watcher workers.
    pub fn init(&self, data_dir: PathBuf, app_handle: AppHandle) {
        *self.app_handle.lock().unwrap() = Some(app_handle.clone());

        let _ = std::fs::create_dir_all(&data_dir);
        let file_path = data_dir.join("downloads_config.json");
        *self.storage_path.lock().unwrap() = Some(file_path.clone());

        let mut folders = Vec::new();

        if file_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(config) = serde_json::from_str::<DownloadsConfig>(&content) {
                    folders = config.watched_folders;
                }
            }
        }

        // Guarantee at least the standard user Downloads folder is configured as default
        if folders.is_empty() {
            if let Some(default_downloads) = get_default_downloads_folder() {
                folders.push(WatchedFolder {
                    id: "default-downloads".to_string(),
                    path: default_downloads.to_string_lossy().to_string(),
                    is_default: true,
                    exists: default_downloads.exists(),
                });
                let _ = self.save_to_disk(&folders);
            }
        } else {
            // Update directory existence state
            for f in folders.iter_mut() {
                f.exists = Path::new(&f.path).exists();
            }
        }

        *self.watched_folders.lock().unwrap() = folders.clone();

        // Start filesystem watchers for all existing folders
        for folder in folders {
            if folder.exists {
                self.start_folder_watcher(&folder.id, &folder.path, app_handle.clone());
            }
        }
    }

    /// Saves the current list of watched folders to disk.
    fn save_to_disk(&self, folders: &[WatchedFolder]) -> Result<(), String> {
        let path_guard = self.storage_path.lock().unwrap();
        if let Some(ref path) = *path_guard {
            let config = DownloadsConfig {
                watched_folders: folders.to_vec(),
            };
            let json = serde_json::to_string_pretty(&config)
                .map_err(|e| format!("Failed to serialize downloads config: {}", e))?;
            std::fs::write(path, json)
                .map_err(|e| format!("Failed to write downloads config to disk: {}", e))?;
        }
        Ok(())
    }

    /// Returns all registered watched folders with freshly verified existence checks.
    pub fn get_watched_folders(&self) -> Vec<WatchedFolder> {
        let mut folders = self.watched_folders.lock().unwrap();
        for f in folders.iter_mut() {
            f.exists = Path::new(&f.path).exists();
        }
        folders.clone()
    }

    /// Adds a new watched folder and immediately starts filesystem monitoring for it.
    pub fn add_watched_folder(&self, path_str: &str) -> Result<WatchedFolder, String> {
        let path = Path::new(path_str);
        if !path.exists() || !path.is_dir() {
            return Err("The specified path is not an accessible directory on disk".to_string());
        }

        let canonical_str = path_str.trim().to_string();
        let mut folders = self.watched_folders.lock().unwrap();

        // Avoid duplicate folder entries
        if folders.iter().any(|f| f.path.eq_ignore_ascii_case(&canonical_str)) {
            return Err("Folder is already in watched folders list".to_string());
        }

        let id = format!("folder_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        let watched = WatchedFolder {
            id: id.clone(),
            path: canonical_str.clone(),
            is_default: false,
            exists: true,
        };

        folders.push(watched.clone());
        self.save_to_disk(&folders)?;

        if let Some(app) = self.app_handle.lock().unwrap().as_ref() {
            self.start_folder_watcher(&id, &canonical_str, app.clone());
        }

        Ok(watched)
    }

    /// Removes a watched folder and cleanly halts its monitoring worker thread.
    pub fn remove_watched_folder(&self, id: &str) -> Result<(), String> {
        let mut folders = self.watched_folders.lock().unwrap();
        let initial_len = folders.len();
        folders.retain(|f| f.id != id);

        if folders.len() == initial_len {
            return Err(format!("Watched folder with id '{}' not found", id));
        }

        self.save_to_disk(&folders)?;

        // Stop the watcher for this folder
        let mut watchers = self.watchers.lock().unwrap();
        if let Some(mut handle) = watchers.remove(id) {
            handle.stop();
        }

        Ok(())
    }

    /// Retrieves current active download state if any.
    pub fn get_current_state(&self) -> Option<DownloadState> {
        if let Some(state) = self.current_state.lock().unwrap().clone() {
            return Some(state);
        }

        let active = self.active_downloads.lock().unwrap();
        if let Some((_, entry)) = active.iter().next() {
            return Some(DownloadState {
                file_name: entry.target_name.clone(),
                progress_percent: None,
                is_completed: false,
                file_path: Some(entry.target_path.to_string_lossy().to_string()),
            });
        }

        None
    }

    /// Starts an asynchronous filesystem watcher for a given folder using Windows ReadDirectoryChangesW.
    fn start_folder_watcher(&self, id: &str, dir_path_str: &str, app_handle: AppHandle) {
        let dir_path = PathBuf::from(dir_path_str);
        if !dir_path.exists() {
            return;
        }

        let stop_signal = Arc::new(AtomicBool::new(false));
        let active_downloads = self.active_downloads.clone();
        let recently_completed = self.recently_completed.clone();

        #[cfg(target_os = "windows")]
        {
            use windows::core::HSTRING;
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::Storage::FileSystem::{
                CreateFileW, ReadDirectoryChangesW, FILE_FLAG_BACKUP_SEMANTICS,
                FILE_LIST_DIRECTORY, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
                OPEN_EXISTING, FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE,
                FILE_NOTIFY_CHANGE_SIZE,
            };

            let hstring_path = HSTRING::from(dir_path_str);
            let raw_handle_res = unsafe {
                CreateFileW(
                    &hstring_path,
                    FILE_LIST_DIRECTORY.0,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    None,
                    OPEN_EXISTING,
                    FILE_FLAG_BACKUP_SEMANTICS,
                    Some(HANDLE::default()),
                )
            };

            let dir_handle = match raw_handle_res {
                Ok(h) => h,
                Err(err) => {
                    eprintln!("Failed to open directory '{}' for watching: {}", dir_path_str, err);
                    return;
                }
            };

            let raw_val = dir_handle.0 as isize;
            let thread_stop = stop_signal.clone();
            let thread_dir_path = dir_path.clone();

            let join_handle = std::thread::spawn(move || {
                let dir_handle = HANDLE(raw_val as *mut _);
                let mut buffer = vec![0u32; 2048]; // 8192 bytes, DWORD-aligned
                let mut last_rename_old_name: Option<String> = None;

                while !thread_stop.load(Ordering::SeqCst) {
                    let mut bytes_returned = 0u32;
                    let result = unsafe {
                        ReadDirectoryChangesW(
                            dir_handle,
                            buffer.as_mut_ptr() as _,
                            (buffer.len() * 4) as u32,
                            false,
                            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_SIZE,
                            Some(&mut bytes_returned),
                            None,
                            None,
                        )
                    };

                    if result.is_err() || thread_stop.load(Ordering::SeqCst) {
                        break;
                    }

                    if bytes_returned == 0 {
                        continue;
                    }

                    // Parse FILE_NOTIFY_INFORMATION chain
                    let mut offset = 0usize;
                    let buf_ptr = buffer.as_ptr() as *const u8;
                    let max_bytes = bytes_returned as usize;

                    while offset < max_bytes {
                        if offset + 12 > max_bytes {
                            break;
                        }

                        let entry_ptr = unsafe { buf_ptr.add(offset) };
                        let next_entry_offset = unsafe { *(entry_ptr as *const u32) } as usize;
                        let action = unsafe { *(entry_ptr.add(4) as *const u32) };
                        let filename_length_bytes = unsafe { *(entry_ptr.add(8) as *const u32) } as usize;

                        if offset + 12 + filename_length_bytes > max_bytes {
                            break;
                        }

                        let filename_chars = filename_length_bytes / 2;
                        let name_ptr = unsafe { entry_ptr.add(12) as *const u16 };
                        let slice = unsafe { std::slice::from_raw_parts(name_ptr, filename_chars) };
                        let file_name = String::from_utf16_lossy(slice);

                        handle_fs_change_event(
                            action,
                            &file_name,
                            &thread_dir_path,
                            &mut last_rename_old_name,
                            &active_downloads,
                            &recently_completed,
                            &app_handle,
                        );

                        if next_entry_offset == 0 {
                            break;
                        }
                        offset += next_entry_offset;
                    }
                }
            });

            let watcher_handle = FolderWatcherHandle {
                stop_signal,
                thread_handle: Some(join_handle),
                raw_dir_handle: raw_val,
            };

            let mut watchers = self.watchers.lock().unwrap();
            watchers.insert(id.to_string(), watcher_handle);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let watcher_handle = FolderWatcherHandle {
                stop_signal,
                thread_handle: None,
            };
            let mut watchers = self.watchers.lock().unwrap();
            watchers.insert(id.to_string(), watcher_handle);
        }
    }
}

impl Drop for DownloadsManager {
    fn drop(&mut self) {
        let mut watchers = self.watchers.lock().unwrap();
        for (_, mut watcher) in watchers.drain() {
            watcher.stop();
        }
    }
}

/// Internal processor for filesystem event actions received from ReadDirectoryChangesW.
fn handle_fs_change_event(
    action: u32,
    file_name: &str,
    dir_path: &Path,
    last_rename_old: &mut Option<String>,
    active_downloads: &Arc<Mutex<HashMap<String, ActiveDownloadEntry>>>,
    recently_completed: &Arc<Mutex<HashMap<String, u64>>>,
    app_handle: &AppHandle,
) {
    const FILE_ACTION_ADDED: u32 = 1;
    const FILE_ACTION_MODIFIED: u32 = 3;
    const FILE_ACTION_RENAMED_OLD_NAME: u32 = 4;
    const FILE_ACTION_RENAMED_NEW_NAME: u32 = 5;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Prune completed cache older than 10 seconds
    {
        let mut completed = recently_completed.lock().unwrap();
        completed.retain(|_, time| now_ms.saturating_sub(*time) < 10_000);
    }

    match action {
        FILE_ACTION_RENAMED_OLD_NAME => {
            *last_rename_old = Some(file_name.to_string());
        }
        FILE_ACTION_RENAMED_NEW_NAME => {
            let old_name = last_rename_old.take();
            let mut is_completion = false;

            // Pattern A: renamed from a recognized partial download name (e.g. "asset.zip.crdownload" -> "asset.zip")
            if let Some(ref old) = old_name {
                if let Some(target) = extract_partial_target_name(old) {
                    if target.eq_ignore_ascii_case(file_name) {
                        is_completion = true;
                    }
                }
            }

            // Pattern B: active downloads map was tracking this target name
            {
                let mut active = active_downloads.lock().unwrap();
                if active.remove(file_name).is_some() {
                    is_completion = true;
                }
            }

            if is_completion {
                let target_path = dir_path.join(file_name);
                let path_key = target_path.to_string_lossy().to_string();

                // Deduplicate completion events within 5 seconds
                let mut completed = recently_completed.lock().unwrap();
                if completed.contains_key(&path_key) {
                    return;
                }
                completed.insert(path_key.clone(), now_ms);

                let state = DownloadState {
                    file_name: file_name.to_string(),
                    progress_percent: None, // Never invent or fabricate percentages
                    is_completed: true,
                    file_path: Some(path_key),
                };

                let _ = app_handle.emit("download.completed", &state);
            }
        }
        FILE_ACTION_ADDED | FILE_ACTION_MODIFIED => {
            // Check if this is a recognized partial download file
            if let Some(target_name) = extract_partial_target_name(file_name) {
                let target_path = dir_path.join(&target_name);
                let path_key = target_path.to_string_lossy().to_string();

                // If this file was already completed recently, ignore lingering write events
                {
                    let completed = recently_completed.lock().unwrap();
                    if completed.contains_key(&path_key) {
                        return;
                    }
                }

                let mut active = active_downloads.lock().unwrap();
                let should_emit = if let Some(entry) = active.get_mut(&target_name) {
                    // Debounce modified events to at most once every 1000ms
                    if now_ms.saturating_sub(entry.last_emitted_ms) >= 1000 {
                        entry.last_emitted_ms = now_ms;
                        true
                    } else {
                        false
                    }
                } else {
                    active.insert(
                        target_name.clone(),
                        ActiveDownloadEntry {
                            target_name: target_name.clone(),
                            target_path: target_path.clone(),
                            last_emitted_ms: now_ms,
                        },
                    );
                    true
                };

                if should_emit {
                    let state = DownloadState {
                        file_name: target_name,
                        progress_percent: None, // Reliable progress signal not provided by browser temp file
                        is_completed: false,
                        file_path: Some(path_key),
                    };

                    let _ = app_handle.emit("download.changed", &state);
                }
            }
        }
        _ => {}
    }
}

/// Returns the default Windows user downloads folder path if available.
pub fn get_default_downloads_folder() -> Option<PathBuf> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(profile).join("Downloads");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Opens the target file's parent folder in Windows File Explorer, or highlights the file if it exists.
pub fn open_folder_or_select_file(target_str: Option<&str>) -> Result<(), String> {
    let path_to_open = if let Some(target) = target_str {
        PathBuf::from(target)
    } else if let Some(default_p) = get_default_downloads_folder() {
        default_p
    } else {
        return Err("No path provided and default downloads folder was not found".to_string());
    };

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer.exe");
        if path_to_open.is_file() {
            cmd.arg(format!("/select,{}", path_to_open.to_string_lossy()));
        } else if path_to_open.exists() {
            cmd.arg(path_to_open.as_os_str());
        } else if let Some(parent) = path_to_open.parent() {
            if parent.exists() {
                cmd.arg(parent.as_os_str());
            } else {
                return Err("Specified download path does not exist on disk".to_string());
            }
        } else {
            return Err("Specified download path does not exist on disk".to_string());
        }

        cmd.spawn()
            .map_err(|e| format!("Failed to spawn explorer.exe: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Opening explorer is only supported on Windows".to_string())
    }
}

/// Displays a native Windows folder picker dialog using IFileOpenDialog with FOS_PICKFOLDERS.
#[cfg(target_os = "windows")]
pub fn pick_folder() -> Result<Option<String>, String> {
    std::thread::spawn(|| {
        use windows::core::HSTRING;
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        };
        use windows::Win32::UI::Shell::{
            FileOpenDialog, IFileOpenDialog, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
        };

        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let dialog: IFileOpenDialog =
                match CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER) {
                    Ok(d) => d,
                    Err(e) => {
                        CoUninitialize();
                        return Err(format!("Failed to initialize FileOpenDialog: {}", e));
                    }
                };

            let title = HSTRING::from("Select Watched Download Folder");
            let _ = dialog.SetTitle(&title);

            use windows::Win32::UI::Shell::FILEOPENDIALOGOPTIONS;

            let options = dialog.GetOptions().unwrap_or(FILEOPENDIALOGOPTIONS(0));
            let _ = dialog.SetOptions(FILEOPENDIALOGOPTIONS(options.0 | FOS_PICKFOLDERS.0));

            if dialog.Show(None).is_err() {
                CoUninitialize();
                return Ok(None);
            }

            let item = match dialog.GetResult() {
                Ok(item) => item,
                Err(e) => {
                    CoUninitialize();
                    return Err(format!("Failed to retrieve folder selection: {}", e));
                }
            };

            let path_pwstr = match item.GetDisplayName(SIGDN_FILESYSPATH) {
                Ok(p) => p,
                Err(e) => {
                    CoUninitialize();
                    return Err(format!("Failed to retrieve folder path: {}", e));
                }
            };

            let path_str = path_pwstr.to_string().map_err(|e| e.to_string())?;
            CoTaskMemFree(Some(path_pwstr.as_ptr() as *const _));

            CoUninitialize();
            Ok(Some(path_str))
        }
    })
    .join()
    .map_err(|_| "Dialog thread panicked".to_string())?
}

#[cfg(not(target_os = "windows"))]
pub fn pick_folder() -> Result<Option<String>, String> {
    Err("Folder picker is only supported on Windows".to_string())
}
