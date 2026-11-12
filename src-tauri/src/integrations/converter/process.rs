//! A short-lived, contained CLI process. No shell, console window, or idle worker.
use std::{
    fs::File,
    io::Read,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};
use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    Win32::{
        Foundation::{
            SetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
        },
        Security::SECURITY_ATTRIBUTES,
        System::{
            JobObjects::*,
            Pipes::CreatePipe,
            SystemServices::JOB_OBJECT_MSG_JOB_MEMORY_LIMIT,
            Threading::*,
            IO::{CreateIoCompletionPort, GetQueuedCompletionStatus},
        },
    },
};

pub const MEMORY_LIMIT: usize = 512 * 1024 * 1024;
const OUTPUT_LIMIT: usize = 1024 * 1024;
const LINE_LIMIT: usize = 64 * 1024;

pub struct ProcessOutput {
    pub code: u32,
    pub stdout: Vec<u8>,
    pub diagnostics: String,
}

fn handle(owned: &OwnedHandle) -> HANDLE {
    HANDLE(owned.as_raw_handle())
}
unsafe fn own(raw: HANDLE) -> OwnedHandle {
    OwnedHandle::from_raw_handle(raw.0)
}

// Windows command-line escaping follows CommandLineToArgvW / the C runtime rules.
fn quote(argument: &str) -> String {
    let mut output = String::from("\"");
    let mut slashes = 0;
    for ch in argument.chars() {
        if ch == '\\' {
            slashes += 1;
            continue;
        }
        output.extend(std::iter::repeat_n(
            '\\',
            if ch == '"' { slashes * 2 + 1 } else { slashes },
        ));
        output.push(ch);
        slashes = 0;
    }
    output.extend(std::iter::repeat_n('\\', slashes * 2));
    output.push('"');
    output
}

fn pipe() -> Result<(OwnedHandle, OwnedHandle), String> {
    let security = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        bInheritHandle: true.into(),
        ..Default::default()
    };
    let (mut read, mut write) = (HANDLE::default(), HANDLE::default());
    unsafe {
        CreatePipe(&mut read, &mut write, Some(&security), 0).map_err(|e| e.to_string())?;
        Ok((own(read), own(write)))
    }
}

struct Attributes {
    pointer: LPPROC_THREAD_ATTRIBUTE_LIST,
    _storage: Vec<usize>,
}
impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.pointer);
        }
    }
}

fn attributes(handles: &[HANDLE]) -> Result<Attributes, String> {
    unsafe {
        let mut bytes = 0;
        let _ = InitializeProcThreadAttributeList(None, 1, Some(0), &mut bytes);
        let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
        let pointer = LPPROC_THREAD_ATTRIBUTE_LIST(storage.as_mut_ptr().cast());
        InitializeProcThreadAttributeList(Some(pointer), 1, Some(0), &mut bytes)
            .map_err(|e| e.to_string())?;
        let result = Attributes {
            pointer,
            _storage: storage,
        };
        UpdateProcThreadAttribute(
            pointer,
            0,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
            Some(handles.as_ptr().cast()),
            size_of_val(handles),
            None,
            None,
        )
        .map_err(|e| e.to_string())?;
        Ok(result)
    }
}

fn read_stdout(mut file: File, overflow: Arc<AtomicBool>) -> Vec<u8> {
    let mut output = Vec::new();
    let mut buffer = [0; 8192];
    while let Ok(count) = file.read(&mut buffer) {
        if count == 0 {
            break;
        }
        if output.len() + count > OUTPUT_LIMIT {
            overflow.store(true, Ordering::Release);
        }
        let keep = count.min(OUTPUT_LIMIT.saturating_sub(output.len()));
        output.extend_from_slice(&buffer[..keep]);
    }
    output
}

fn read_stderr(mut file: File, sender: mpsc::SyncSender<f64>, overflow: Arc<AtomicBool>) -> String {
    let mut line = Vec::new();
    let mut diagnostics = Vec::new();
    let mut buffer = [0; 4096];
    while let Ok(count) = file.read(&mut buffer) {
        if count == 0 {
            break;
        }
        for byte in &buffer[..count] {
            if *byte == b'\n' {
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&line) {
                    if value["event"] == "progress" {
                        if let Some(fraction) = value["fraction"].as_f64().filter(|n| n.is_finite())
                        {
                            let _ = sender.try_send(fraction.clamp(0.0, 1.0));
                        }
                    }
                }
                line.clear();
            } else if line.len() < LINE_LIMIT {
                line.push(*byte);
            } else {
                overflow.store(true, Ordering::Release);
            }
        }
        diagnostics.extend_from_slice(&buffer[..count]);
        if diagnostics.len() > LINE_LIMIT {
            diagnostics.drain(..diagnostics.len() - LINE_LIMIT);
        }
    }
    String::from_utf8_lossy(&diagnostics).into_owned()
}

pub fn run(
    executable: &Path,
    arguments: &[String],
    timeout: Duration,
    cancel: &AtomicBool,
    mut progress: impl FnMut(f64),
) -> Result<ProcessOutput, String> {
    if cancel.load(Ordering::Acquire) {
        return Err("Conversion cancelled.".into());
    }
    unsafe {
        let job = own(CreateJobObjectW(None, PCWSTR::null()).map_err(|e| e.to_string())?);
        let limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
            BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                LimitFlags: JOB_OBJECT_LIMIT_JOB_MEMORY | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                ..Default::default()
            },
            JobMemoryLimit: MEMORY_LIMIT,
            ..Default::default()
        };
        SetInformationJobObject(
            handle(&job),
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of_val(&limits) as u32,
        )
        .map_err(|_| "Could not enforce the converter memory limit.".to_string())?;
        let completion =
            own(CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, 1)
                .map_err(|e| e.to_string())?);
        let association = JOBOBJECT_ASSOCIATE_COMPLETION_PORT {
            CompletionKey: 1usize as *mut _,
            CompletionPort: handle(&completion),
        };
        SetInformationJobObject(
            handle(&job),
            JobObjectAssociateCompletionPortInformation,
            (&association as *const JOBOBJECT_ASSOCIATE_COMPLETION_PORT).cast(),
            size_of_val(&association) as u32,
        )
        .map_err(|e| e.to_string())?;

        let (stdout_read, stdout_write) = pipe()?;
        let (stderr_read, stderr_write) = pipe()?;
        let (stdin_read, stdin_write) = pipe()?;
        for read in [&stdout_read, &stderr_read, &stdin_write] {
            SetHandleInformation(handle(read), HANDLE_FLAG_INHERIT.0, Default::default())
                .map_err(|e| e.to_string())?;
        }
        let inherited = [
            handle(&stdout_write),
            handle(&stderr_write),
            handle(&stdin_read),
        ];
        let attributes = attributes(&inherited)?;
        let startup = STARTUPINFOEXW {
            StartupInfo: STARTUPINFOW {
                cb: size_of::<STARTUPINFOEXW>() as u32,
                dwFlags: STARTF_USESTDHANDLES,
                hStdOutput: inherited[0],
                hStdError: inherited[1],
                hStdInput: inherited[2],
                ..Default::default()
            },
            lpAttributeList: attributes.pointer,
        };
        let executable_text = HSTRING::from(executable.as_os_str());
        let text = std::iter::once(executable.to_string_lossy().into_owned())
            .chain(arguments.iter().cloned())
            .map(|arg| quote(&arg))
            .collect::<Vec<_>>()
            .join(" ");
        if text.contains('\0') {
            return Err("A converter argument contains an invalid character.".into());
        }
        let mut command: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let mut information = PROCESS_INFORMATION::default();
        CreateProcessW(
            &executable_text,
            Some(PWSTR(command.as_mut_ptr())),
            None,
            None,
            true,
            CREATE_SUSPENDED
                | CREATE_NO_WINDOW
                | EXTENDED_STARTUPINFO_PRESENT
                | BELOW_NORMAL_PRIORITY_CLASS,
            None,
            PCWSTR::null(),
            &startup.StartupInfo,
            &mut information,
        )
        .map_err(|e| format!("Could not start Filee: {e}"))?;
        let process = own(information.hProcess);
        let main_thread = own(information.hThread);
        if let Err(error) = AssignProcessToJobObject(handle(&job), handle(&process)) {
            let _ = TerminateProcess(handle(&process), 1);
            return Err(format!("Could not contain the converter: {error}"));
        }
        if ResumeThread(handle(&main_thread)) == u32::MAX {
            return Err("Could not resume the converter.".into());
        }
        drop(stdout_write);
        drop(stderr_write);
        drop(stdin_read);
        drop(stdin_write);
        let overflow = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::sync_channel(8);
        let stdout_overflow = overflow.clone();
        let stderr_overflow = overflow.clone();
        let stdout = thread::spawn(move || read_stdout(File::from(stdout_read), stdout_overflow));
        let stderr =
            thread::spawn(move || read_stderr(File::from(stderr_read), sender, stderr_overflow));
        let start = Instant::now();
        let mut failure = None;
        loop {
            for fraction in receiver.try_iter() {
                progress(fraction);
            }
            let (mut message, mut key, mut overlapped) = (0, 0, std::ptr::null_mut());
            while GetQueuedCompletionStatus(
                handle(&completion),
                &mut message,
                &mut key,
                &mut overlapped,
                0,
            )
            .is_ok()
            {
                if message == JOB_OBJECT_MSG_JOB_MEMORY_LIMIT {
                    failure = Some(
                        "This file exceeded the 512 MiB converter limit. Try a smaller file."
                            .to_string(),
                    );
                }
            }
            if cancel.load(Ordering::Acquire) {
                failure = Some("Conversion cancelled.".into());
            }
            if start.elapsed() >= timeout {
                failure =
                    Some("Filee took too long. Try a smaller file or check its engines.".into());
            }
            if overflow.load(Ordering::Acquire) {
                failure =
                    Some("Filee returned more output than the converter can safely hold.".into());
            }
            if failure.is_some() || WaitForSingleObject(handle(&process), 25) == WAIT_OBJECT_0 {
                break;
            }
        }
        // Also ends descendants that kept a pipe open after the CLI exited.
        let _ = TerminateJobObject(handle(&job), 1);
        let _ = WaitForSingleObject(handle(&process), 2000);
        let mut code = 1;
        let _ = GetExitCodeProcess(handle(&process), &mut code);
        let output = stdout
            .join()
            .map_err(|_| "Could not read Filee's result.".to_string())?;
        let diagnostics = stderr
            .join()
            .map_err(|_| "Could not read Filee's status.".to_string())?;
        if overflow.load(Ordering::Acquire) && failure.is_none() {
            failure = Some("Filee's output exceeded the safety limit.".into());
        }
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(ProcessOutput {
            code,
            stdout: output,
            diagnostics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arguments_round_trip_through_windows_parser() {
        use windows::Win32::{Foundation::LocalFree, UI::Shell::CommandLineToArgvW};
        let arguments = [
            "app.exe",
            "",
            "C:\\a folder\\",
            "a\"b",
            "a\\\"b",
            "$(literal); & text",
            "日本語",
        ];
        let text = HSTRING::from(arguments.map(quote).join(" "));
        unsafe {
            let mut count = 0;
            let parsed = CommandLineToArgvW(&text, &mut count);
            assert!(!parsed.is_null());
            let values = std::slice::from_raw_parts(parsed, count as usize)
                .iter()
                .map(|s| s.to_string().unwrap())
                .collect::<Vec<_>>();
            let _ = LocalFree(Some(windows::Win32::Foundation::HLOCAL(parsed.cast())));
            assert_eq!(values, arguments);
        }
    }
}
