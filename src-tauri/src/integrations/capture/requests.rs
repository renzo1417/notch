//! Request-scoped cancellation survives scheduling on a blocking worker. A
//! completed request's late cancel can never target a subsequent screenshot.
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

const RETAINED_IDS: usize = 64;

pub(super) struct CaptureRequest {
    pub id: String,
    pub cancelled: AtomicBool,
    pub overlay: AtomicUsize,
}

#[derive(Default)]
struct RequestState {
    active: Option<Arc<CaptureRequest>>,
    cancelled_before_start: VecDeque<String>,
    completed: VecDeque<String>,
}

#[derive(Default)]
pub(super) struct CaptureRequests {
    state: Mutex<RequestState>,
}

impl CaptureRequests {
    /// None means the request was cancelled before its worker began.
    pub fn begin(&self, id: &str) -> Result<Option<Arc<CaptureRequest>>, String> {
        validate_id(id)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Screenshot request state is unavailable.")?;
        if state.completed.iter().any(|completed| completed == id) {
            return Err("This screenshot request has already finished.".into());
        }
        if let Some(index) = state
            .cancelled_before_start
            .iter()
            .position(|cancelled| cancelled == id)
        {
            state.cancelled_before_start.remove(index);
            retain_id(&mut state.completed, id);
            return Ok(None);
        }
        if state.active.is_some() {
            return Err("A screenshot is already in progress.".into());
        }
        let request = Arc::new(CaptureRequest {
            id: id.to_string(),
            cancelled: AtomicBool::new(false),
            overlay: AtomicUsize::new(0),
        });
        state.active = Some(request.clone());
        Ok(Some(request))
    }

    /// Returns only this request's active overlay state. Unknown IDs are kept
    /// briefly so cancellation delivered before start remains effective.
    pub fn cancel(&self, id: &str) -> Result<Option<Arc<CaptureRequest>>, String> {
        validate_id(id)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Screenshot request state is unavailable.")?;
        if let Some(active) = state.active.as_ref().filter(|active| active.id == id) {
            active.cancelled.store(true, Ordering::SeqCst);
            return Ok(Some(active.clone()));
        }
        if !state.completed.iter().any(|completed| completed == id) {
            retain_id(&mut state.cancelled_before_start, id);
        }
        Ok(None)
    }

    pub fn finish(&self, id: &str) {
        if let Ok(mut state) = self.state.lock() {
            if state.active.as_ref().is_some_and(|active| active.id == id) {
                state.active = None;
                retain_id(&mut state.completed, id);
            }
        }
    }
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
    {
        return Err("Invalid screenshot request identifier.".into());
    }
    Ok(())
}

fn retain_id(ids: &mut VecDeque<String>, id: &str) {
    if !ids.iter().any(|existing| existing == id) {
        if ids.len() == RETAINED_IDS {
            ids.pop_front();
        }
        ids.push_back(id.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_before_worker_start_is_not_reset() {
        let requests = CaptureRequests::default();
        assert!(requests.cancel("queued-1").unwrap().is_none());
        assert!(requests.begin("queued-1").unwrap().is_none());
        assert!(requests.begin("queued-1").is_err());
        assert!(requests.begin("next-2").unwrap().is_some());
    }

    #[test]
    fn late_cancel_does_not_cancel_next_request() {
        let requests = CaptureRequests::default();
        let first = requests.begin("first").unwrap().unwrap();
        first.overlay.store(11, Ordering::SeqCst);
        requests.finish("first");
        let second = requests.begin("second").unwrap().unwrap();
        second.overlay.store(22, Ordering::SeqCst);
        assert!(requests.cancel("first").unwrap().is_none());
        assert!(!first.cancelled.load(Ordering::SeqCst));
        assert!(!second.cancelled.load(Ordering::SeqCst));
        assert_eq!(second.overlay.load(Ordering::SeqCst), 22);
        let cancelled = requests.cancel("second").unwrap().unwrap();
        assert!(Arc::ptr_eq(&second, &cancelled));
        assert!(second.cancelled.load(Ordering::SeqCst));
    }

    #[test]
    fn active_request_excludes_another_and_pending_ids_are_bounded() {
        let requests = CaptureRequests::default();
        assert!(requests.begin("first").unwrap().is_some());
        assert!(requests.begin("second").is_err());
        for index in 0..128 {
            requests.cancel(&format!("queued-{index}")).unwrap();
        }
        assert_eq!(
            requests.state.lock().unwrap().cancelled_before_start.len(),
            RETAINED_IDS
        );
        requests.finish("first");
        assert!(requests.begin("second").unwrap().is_some());
        assert!(requests.cancel("bad id").is_err());
    }
}
