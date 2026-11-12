// Tests the real Windows process boundary without starting the Tauri UI runtime.
#[path = "../src/integrations/converter/files.rs"]
mod files;
#[path = "../src/integrations/converter/process.rs"]
mod process;
use files::Staging;
#[path = "../src/integrations/converter/protocol.rs"]
mod protocol;
use protocol::parse_result;
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::Duration,
};
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_inspection_deduplicates_and_rejects_directories() {
        assert!(files::TARGETS.contains(&"pdf"));
        let parent = std::env::temp_dir().join(format!("notch-inspect-{}", files::identity()));
        std::fs::create_dir(&parent).unwrap();
        let source = parent.join("photo.png");
        std::fs::write(&source, b"source").unwrap();
        let text = source.to_string_lossy().into_owned();
        let inspected = files::inspect(vec![text.clone(), text]).unwrap();
        assert_eq!(inspected.len(), 1);
        assert_eq!(inspected[0].bytes, 6);
        assert!(files::inspect(vec![parent.to_string_lossy().into()]).is_err());
        std::fs::remove_file(source).unwrap();
        std::fs::remove_dir(parent).unwrap();
    }

    fn fixture() -> &'static Path {
        static FIXTURE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        FIXTURE.get_or_init(|| {
            let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
            let folder = manifest.join("target").join("converter-fixture");
            std::fs::create_dir_all(&folder).unwrap();
            let executable = folder.join("filee-cli.exe");
            let status = std::process::Command::new("rustc")
                .arg("--edition=2021")
                .arg(manifest.join("../tests/fixtures/filee-cli.rs"))
                .arg("-o")
                .arg(&executable)
                .status()
                .unwrap();
            assert!(status.success());
            executable
        })
    }

    fn fixture_conversion(
        name: &str,
        timeout: Duration,
        cancel: &AtomicBool,
    ) -> (Staging, Result<process::ProcessOutput, String>) {
        let parent = std::env::temp_dir().join(format!("notch-test-{}", files::identity()));
        std::fs::create_dir(&parent).unwrap();
        let stage = Staging::new(&parent).unwrap();
        let source = stage.path.join(format!("{name}.png"));
        std::fs::write(&source, b"unchanged source").unwrap();
        let result = process::run(
            fixture(),
            &[
                "convert".into(),
                source.to_string_lossy().into(),
                "--to".into(),
                "png".into(),
                "--json".into(),
                "-o".into(),
                stage.path.to_string_lossy().into(),
            ],
            timeout,
            cancel,
            |_| {},
        );
        assert_eq!(std::fs::read(source).unwrap(), b"unchanged source");
        (stage, result)
    }

    #[test]
    fn contained_cli_contract_supports_probe_progress_and_final_json() {
        let output = process::run(
            fixture(),
            &["version".into()],
            Duration::from_secs(5),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).starts_with("filee "));
        let (stage, result) =
            fixture_conversion("photo", Duration::from_secs(5), &AtomicBool::new(false));
        let result = result.unwrap();
        assert!(result.diagnostics.contains("progress"));
        let outputs = parse_result(&result.stdout, result.code).unwrap();
        assert!(Path::new(&outputs[0]).is_file());
        let parent = stage.path.parent().unwrap().to_owned();
        drop(stage);
        let _ = std::fs::remove_dir(parent);
    }

    #[test]
    fn oversized_cli_output_is_stopped() {
        let (stage, result) =
            fixture_conversion("noisy", Duration::from_secs(5), &AtomicBool::new(false));
        assert!(result.err().unwrap().contains("output"));
        let parent = stage.path.parent().unwrap().to_owned();
        drop(stage);
        let _ = std::fs::remove_dir(parent);
    }

    #[test]
    fn whole_converter_job_obeys_the_memory_limit() {
        let (stage, result) =
            fixture_conversion("memory", Duration::from_secs(10), &AtomicBool::new(false));
        assert!(result.err().unwrap().contains("512 MiB"));
        let parent = stage.path.parent().unwrap().to_owned();
        drop(stage);
        let _ = std::fs::remove_dir(parent);
    }

    #[test]
    fn timeout_terminates_descendants_and_cleanup_removes_partial_outputs() {
        use std::os::windows::io::FromRawHandle;
        use windows::Win32::{
            Foundation::WAIT_OBJECT_0,
            System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE},
        };
        let (stage, result) =
            fixture_conversion("slow", Duration::from_millis(1200), &AtomicBool::new(false));
        assert!(result.err().unwrap().contains("too long"));
        let pid: u32 = std::fs::read_to_string(stage.path.join("child.pid"))
            .unwrap()
            .parse()
            .unwrap();
        unsafe {
            if let Ok(process) = OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
                let _owned = std::os::windows::io::OwnedHandle::from_raw_handle(process.0);
                assert_eq!(WaitForSingleObject(process, 1000), WAIT_OBJECT_0);
            }
        }
        let path = stage.path.clone();
        let parent = path.parent().unwrap().to_owned();
        drop(stage);
        assert!(!path.exists());
        let _ = std::fs::remove_dir(parent);
    }

    #[test]
    fn cancellation_is_checked_before_starting_a_process() {
        let result = process::run(
            fixture(),
            &["version".into()],
            Duration::from_secs(1),
            &AtomicBool::new(true),
            |_| {},
        );
        assert!(result.err().unwrap().contains("cancelled"));
    }

    #[test]
    fn active_cancellation_stops_the_cli_and_its_children() {
        use std::sync::{atomic::Ordering, Arc};
        let _ = fixture();
        let parent = std::env::temp_dir().join(format!("notch-cancel-{}", files::identity()));
        std::fs::create_dir(&parent).unwrap();
        let stage = Staging::new(&parent).unwrap();
        let source = stage.path.join("slow.png");
        std::fs::write(&source, b"source").unwrap();
        let ready = stage.path.join("child.pid");
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let cancel_thread = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            while !ready.exists() && start.elapsed() < Duration::from_secs(5) {
                std::thread::sleep(Duration::from_millis(10));
            }
            let started = ready.exists();
            worker_cancel.store(true, Ordering::Release);
            started
        });
        let result = process::run(
            fixture(),
            &[
                "convert".into(),
                source.to_string_lossy().into(),
                "--to".into(),
                "png".into(),
                "--json".into(),
                "-o".into(),
                stage.path.to_string_lossy().into(),
            ],
            Duration::from_secs(8),
            &cancel,
            |_| {},
        );
        assert!(
            cancel_thread.join().unwrap(),
            "child readiness was observed before cancelling"
        );
        assert!(result.err().unwrap().contains("cancelled"));
        drop(stage);
        let _ = std::fs::remove_dir(parent);
    }
}
