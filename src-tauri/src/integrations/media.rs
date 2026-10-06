//! Media Integration module for Windows Media Session (GSMTC).
//!
//! Uses Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager
//! to provide native, event-driven media session monitoring and playback controls.
//!
//! Defined in PRODUCT_SPEC.md section 13 and Phase 3 requirements.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(target_os = "windows")]
use windows::Foundation::TypedEventHandler;
#[cfg(target_os = "windows")]
use windows::Media::Control::{
    CurrentSessionChangedEventArgs,
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionMediaProperties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    MediaPropertiesChangedEventArgs,
    PlaybackInfoChangedEventArgs,
    TimelinePropertiesChangedEventArgs,
};
#[cfg(target_os = "windows")]
use windows::Storage::Streams::DataReader;
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

/// Media transport controls capabilities exposed by the active session.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MediaCapabilities {
    pub can_play: bool,
    pub can_pause: bool,
    pub can_toggle_play_pause: bool,
    pub can_skip_previous: bool,
    pub can_skip_next: bool,
}

/// Compact typed media state model returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    pub has_session: bool,
    pub is_playing: bool,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub artwork_data_url: Option<String>,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub capabilities: MediaCapabilities,
    pub source_app_id: Option<String>,
}

#[cfg(target_os = "windows")]
struct SessionSubscription {
    session: GlobalSystemMediaTransportControlsSession,
    media_props_token: Option<i64>,
    playback_info_token: Option<i64>,
    timeline_token: Option<i64>,
}

#[cfg(target_os = "windows")]
impl SessionSubscription {
    fn unregister(&self) {
        if let Some(token) = self.media_props_token {
            let _ = self.session.RemoveMediaPropertiesChanged(token);
        }
        if let Some(token) = self.playback_info_token {
            let _ = self.session.RemovePlaybackInfoChanged(token);
        }
        if let Some(token) = self.timeline_token {
            let _ = self.session.RemoveTimelinePropertiesChanged(token);
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for SessionSubscription {
    fn drop(&mut self) {
        self.unregister();
    }
}

/// Central manager coordinating Windows GSMTC session monitoring and playback.
pub struct MediaManager {
    app_handle: OnceLock<tauri::AppHandle>,
    cached_state: Mutex<MediaState>,
    #[cfg(target_os = "windows")]
    session_manager: Mutex<Option<GlobalSystemMediaTransportControlsSessionManager>>,
    #[cfg(target_os = "windows")]
    manager_token: Mutex<Option<i64>>,
    #[cfg(target_os = "windows")]
    current_subscription: Mutex<Option<SessionSubscription>>,
}

impl Default for MediaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaManager {
    pub fn new() -> Self {
        Self {
            app_handle: OnceLock::new(),
            cached_state: Mutex::new(MediaState::default()),
            #[cfg(target_os = "windows")]
            session_manager: Mutex::new(None),
            #[cfg(target_os = "windows")]
            manager_token: Mutex::new(None),
            #[cfg(target_os = "windows")]
            current_subscription: Mutex::new(None),
        }
    }

    /// Sets the app handle used for event emission.
    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        let _ = self.app_handle.set(handle);
    }

    /// Returns the current cached media state.
    pub fn get_state(&self) -> MediaState {
        self.cached_state
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Initializes GSMTC session manager and registers native event listeners on a worker thread.
    #[allow(unused_variables)]
    pub fn init(self: &Arc<Self>, handle: tauri::AppHandle) {
        self.set_app_handle(handle);

        #[cfg(target_os = "windows")]
        {
            let this = Arc::clone(self);
            let _ = std::thread::Builder::new()
                .name("notch-media-listener".to_string())
                .spawn(move || {
                    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
                    this.setup_session_manager();
                });
        }
    }

    #[cfg(target_os = "windows")]
    fn setup_session_manager(self: &Arc<Self>) {
        let op = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            Ok(op) => op,
            Err(e) => {
                eprintln!("[MediaManager] GSMTC SessionManager RequestAsync failed: {e}");
                return;
            }
        };

        let manager = match op.get() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[MediaManager] GSMTC SessionManager RequestAsync get() failed: {e}");
                return;
            }
        };

        // Register CurrentSessionChanged callback
        let this = Arc::clone(self);
        let token = manager.CurrentSessionChanged(&TypedEventHandler::<
            GlobalSystemMediaTransportControlsSessionManager,
            CurrentSessionChangedEventArgs,
        >::new(move |_sender, _args| {
            this.on_current_session_changed();
            Ok(())
        }));

        match token {
            Ok(t) => {
                *self.manager_token.lock().unwrap() = Some(t);
            }
            Err(e) => {
                eprintln!("[MediaManager] Failed to register CurrentSessionChanged: {e}");
            }
        }

        *self.session_manager.lock().unwrap() = Some(manager.clone());

        // Perform initial active session scan
        self.update_active_session(&manager);
    }

    #[cfg(target_os = "windows")]
    fn on_current_session_changed(self: &Arc<Self>) {
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let manager = {
            let lock = self.session_manager.lock().unwrap();
            lock.clone()
        };

        if let Some(mgr) = manager {
            self.update_active_session(&mgr);
        }
    }

    #[cfg(target_os = "windows")]
    fn update_active_session(self: &Arc<Self>, manager: &GlobalSystemMediaTransportControlsSessionManager) {
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

        // Unregister previous session listeners
        {
            let mut sub_lock = self.current_subscription.lock().unwrap();
            if let Some(old_sub) = sub_lock.take() {
                old_sub.unregister();
            }
        }

        match manager.GetCurrentSession() {
            Ok(session) => {
                let this_props = Arc::clone(self);
                let props_token = session.MediaPropertiesChanged(&TypedEventHandler::<
                    GlobalSystemMediaTransportControlsSession,
                    MediaPropertiesChangedEventArgs,
                >::new(move |_sender, _args| {
                    this_props.refresh_properties();
                    Ok(())
                })).ok();

                let this_playback = Arc::clone(self);
                let playback_token = session.PlaybackInfoChanged(&TypedEventHandler::<
                    GlobalSystemMediaTransportControlsSession,
                    PlaybackInfoChangedEventArgs,
                >::new(move |_sender, _args| {
                    this_playback.refresh_playback_info();
                    Ok(())
                })).ok();

                let this_timeline = Arc::clone(self);
                let timeline_token = session.TimelinePropertiesChanged(&TypedEventHandler::<
                    GlobalSystemMediaTransportControlsSession,
                    TimelinePropertiesChangedEventArgs,
                >::new(move |_sender, _args| {
                    this_timeline.refresh_timeline();
                    Ok(())
                })).ok();

                let sub = SessionSubscription {
                    session: session.clone(),
                    media_props_token: props_token,
                    playback_info_token: playback_token,
                    timeline_token,
                };

                *self.current_subscription.lock().unwrap() = Some(sub);

                let state = read_full_session_state(&session);
                self.commit_and_emit_state(state);
            }
            Err(_) => {
                // No current media session
                self.commit_and_emit_state(MediaState::default());
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn refresh_properties(&self) {
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let session = {
            let lock = self.current_subscription.lock().unwrap();
            lock.as_ref().map(|s| s.session.clone())
        };

        if let Some(session) = session {
            let state = read_full_session_state(&session);
            self.commit_and_emit_state(state);
        }
    }

    #[cfg(target_os = "windows")]
    fn refresh_playback_info(&self) {
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let session = {
            let lock = self.current_subscription.lock().unwrap();
            lock.as_ref().map(|s| s.session.clone())
        };

        if let Some(session) = session {
            let state = read_full_session_state(&session);
            self.commit_and_emit_state(state);
        }
    }

    #[cfg(target_os = "windows")]
    fn refresh_timeline(&self) {
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let session = {
            let lock = self.current_subscription.lock().unwrap();
            lock.as_ref().map(|s| s.session.clone())
        };

        if let Some(session) = session {
            if let Ok(timeline) = session.GetTimelineProperties() {
                let position_ms = timeline.Position().ok().map(|p| (p.Duration.max(0) as u64) / 10_000);
                let duration_ms = timeline.EndTime().ok().map(|e| (e.Duration.max(0) as u64) / 10_000);

                let mut lock = self.cached_state.lock().unwrap();
                if lock.position_ms != position_ms || lock.duration_ms != duration_ms {
                    lock.position_ms = position_ms;
                    lock.duration_ms = duration_ms;
                    let state_clone = lock.clone();
                    drop(lock);

                    if let Some(app) = self.app_handle.get() {
                        use tauri::Emitter;
                        let _ = app.emit("media-state-changed", &state_clone);
                    }
                }
            }
        }
    }


    fn commit_and_emit_state(&self, new_state: MediaState) {
        let mut lock = self.cached_state.lock().unwrap();
        if *lock != new_state {
            *lock = new_state.clone();
            drop(lock);

            if let Some(app) = self.app_handle.get() {
                use tauri::Emitter;
                let _ = app.emit("media-state-changed", &new_state);
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn get_active_session_copy(&self) -> Result<GlobalSystemMediaTransportControlsSession, String> {
        let lock = self.current_subscription.lock().unwrap();
        lock.as_ref()
            .map(|s| s.session.clone())
            .ok_or_else(|| "No active Windows media session available".to_string())
    }

    /// Toggle playback state (Play / Pause).
    pub fn toggle_play_pause(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            let session = self.get_active_session_copy()?;

            let is_currently_playing = {
                let state = self.cached_state.lock().unwrap();
                state.is_playing
            };

            // Attempt TryTogglePlayPauseAsync first
            let toggled = session
                .TryTogglePlayPauseAsync()
                .map_err(|e| e.to_string())?
                .get()
                .unwrap_or(false);

            if !toggled {
                // Explicit fallback if toggle capability is not natively acknowledged
                if is_currently_playing {
                    let _ = session.TryPauseAsync().map_err(|e| e.to_string())?.get();
                } else {
                    let _ = session.TryPlayAsync().map_err(|e| e.to_string())?.get();
                }
            }

            // Immediately refresh playback state
            self.refresh_playback_info();
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        Err("Media session controls only available on Windows".to_string())
    }

    /// Skip to previous track.
    pub fn skip_previous(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            let session = self.get_active_session_copy()?;

            let _ = session
                .TrySkipPreviousAsync()
                .map_err(|e| e.to_string())?
                .get()
                .map_err(|e| e.to_string())?;

            self.refresh_properties();
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        Err("Media session controls only available on Windows".to_string())
    }

    /// Skip to next track.
    pub fn skip_next(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            let session = self.get_active_session_copy()?;

            let _ = session
                .TrySkipNextAsync()
                .map_err(|e| e.to_string())?
                .get()
                .map_err(|e| e.to_string())?;

            self.refresh_properties();
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        Err("Media session controls only available on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
impl Drop for MediaManager {
    fn drop(&mut self) {
        if let (Some(manager), Some(token)) = (
            self.session_manager.lock().unwrap().as_ref(),
            *self.manager_token.lock().unwrap(),
        ) {
            let _ = manager.RemoveCurrentSessionChanged(token);
        }
    }
}

#[cfg(target_os = "windows")]
fn read_full_session_state(session: &GlobalSystemMediaTransportControlsSession) -> MediaState {
    let source_app_id = session
        .SourceAppUserModelId()
        .ok()
        .map(|s| s.to_string_lossy())
        .filter(|s| !s.trim().is_empty());

    let mut is_playing = false;
    let mut capabilities = MediaCapabilities::default();

    if let Ok(playback_info) = session.GetPlaybackInfo() {
        let status = playback_info
            .PlaybackStatus()
            .unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed);

        is_playing = status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

        if let Ok(controls) = playback_info.Controls() {
            let can_play = controls.IsPlayEnabled().unwrap_or(false);
            let can_pause = controls.IsPauseEnabled().unwrap_or(false);
            let can_toggle = controls.IsPlayPauseToggleEnabled().unwrap_or(false);

            capabilities = MediaCapabilities {
                can_play,
                can_pause,
                can_toggle_play_pause: can_toggle || can_play || can_pause,
                can_skip_previous: controls.IsPreviousEnabled().unwrap_or(false),
                can_skip_next: controls.IsNextEnabled().unwrap_or(false),
            };
        }
    }

    let mut title: Option<String> = None;
    let mut artist: Option<String> = None;
    let mut album: Option<String> = None;
    let mut artwork_data_url: Option<String> = None;

    if let Ok(props_op) = session.TryGetMediaPropertiesAsync() {
        if let Ok(props) = props_op.get() {
            title = props
                .Title()
                .ok()
                .map(|s| s.to_string_lossy())
                .filter(|s| !s.trim().is_empty());

            artist = props
                .Artist()
                .ok()
                .map(|s| s.to_string_lossy())
                .filter(|s| !s.trim().is_empty());

            album = props
                .AlbumTitle()
                .ok()
                .map(|s| s.to_string_lossy())
                .filter(|s| !s.trim().is_empty());

            artwork_data_url = extract_artwork_data_url(&props);
        }
    }

    let mut position_ms: Option<u64> = None;
    let mut duration_ms: Option<u64> = None;

    if let Ok(timeline) = session.GetTimelineProperties() {
        position_ms = timeline
            .Position()
            .ok()
            .map(|p| (p.Duration.max(0) as u64) / 10_000);
        duration_ms = timeline
            .EndTime()
            .ok()
            .map(|e| (e.Duration.max(0) as u64) / 10_000);
    }

    MediaState {
        has_session: true,
        is_playing,
        title,
        artist,
        album,
        artwork_data_url,
        position_ms,
        duration_ms,
        capabilities,
        source_app_id,
    }
}

#[cfg(target_os = "windows")]
fn extract_artwork_data_url(
    props: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Option<String> {
    let thumb_ref = props.Thumbnail().ok()?;
    let stream_op = thumb_ref.OpenReadAsync().ok()?;
    let stream = stream_op.get().ok()?;
    let size = stream.Size().ok()?;

    // Reject empty streams or abnormally large artwork (> 2MB)
    if size == 0 || size > 2 * 1024 * 1024 {
        return None;
    }

    let reader = DataReader::CreateDataReader(&stream).ok()?;
    let load_op = reader.LoadAsync(size as u32).ok()?;
    load_op.get().ok()?;

    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes).ok()?;

    let content_type = stream
        .ContentType()
        .ok()
        .map(|s| s.to_string_lossy())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "image/jpeg".to_string());

    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", content_type, encoded))
}
