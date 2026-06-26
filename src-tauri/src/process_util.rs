use crate::emit_progress_from;
use crate::job_control::{wait_active_child, JobController, CANCELLED_BY_USER};
use crate::progress::{parse_progress_line, PhaseProgress, ProgressKind};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Manager};

pub fn find_executable(name: &str) -> Option<PathBuf> {
    let candidates = if cfg!(windows) {
        vec![format!("{name}.exe"), name.to_string()]
    } else {
        vec![name.to_string()]
    };

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for candidate in &candidates {
                let full = dir.join(candidate);
                if full.is_file() {
                    return Some(full);
                }
            }
        }
    }

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

/// Run a short command and capture stdout. Stderr is discarded to avoid pipe deadlocks.
pub fn run_command_output(program: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("Failed to run {}: {e}", program.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(if stderr.is_empty() {
            format!("Command failed (exit {:?}): {stdout}", output.status.code())
        } else {
            format!("Command failed: {stderr}")
        })
    }
}

fn drain_pipe_to_string<R: std::io::Read + Send + 'static>(pipe: R) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let reader = BufReader::new(pipe);
        reader
            .lines()
            .map_while(Result::ok)
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn is_meaningless_stream(s: &str) -> bool {
    let t = s.trim();
    t.is_empty()
        || t.eq_ignore_ascii_case("null")
        || t == "{}"
        || t == "[]"
}

fn format_command_failure(stderr: &str, stdout: &str, exit_code: Option<i32>) -> String {
    let stderr_trimmed = stderr.trim();
    if !is_meaningless_stream(stderr_trimmed) {
        return format!("Command failed: {stderr_trimmed}");
    }
    let stdout_trimmed = stdout.trim();
    if !is_meaningless_stream(stdout_trimmed) {
        return format!("Command failed: {stdout_trimmed}");
    }
    match exit_code {
        Some(code) => format!("Command failed with exit code {code}"),
        None => "Command failed".into(),
    }
}

/// Run a short command with cancellation support (used for metadata fetch).
pub fn run_command_output_cancellable(
    app: &AppHandle,
    program: &Path,
    args: &[&str],
) -> Result<String, String> {
    let jobs = app.state::<JobController>();

    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {e}", program.display()))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stdout_handle = stdout.map(drain_pipe_to_string);
    let stderr_handle = stderr.map(drain_pipe_to_string);

    jobs.set_child(child);

    let status = wait_active_child(&jobs)?;

    if jobs.is_cancelled() {
        return Err(CANCELLED_BY_USER.into());
    }

    let stdout_str = stdout_handle
        .map(|h| h.join().unwrap_or_default())
        .unwrap_or_default();
    let stderr_str = stderr_handle
        .map(|h| h.join().unwrap_or_default())
        .unwrap_or_default();

    if status.success() {
        Ok(stdout_str)
    } else {
        Err(format_command_failure(
            &stderr_str,
            &stdout_str,
            status.code(),
        ))
    }
}

/// Run a long command, parse stderr for progress, emit UI updates, return stdout.
pub fn run_command_with_progress(
    app: &AppHandle,
    program: &Path,
    args: &[&str],
    kind: ProgressKind,
    progress: &PhaseProgress<'_>,
) -> Result<String, String> {
    let jobs = app.state::<JobController>();

    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {e}", program.display()))?;

    let stderr = child
        .stderr
        .take()
        .ok_or("Failed to capture stderr")?;
    let stdout = child.stdout.take();

    jobs.set_child(child);

    let (tx_err, rx_err) = mpsc::channel();

    let progress_app = progress.app.clone();
    let progress_start = progress.start;
    let progress_end = progress.end;

    let stderr_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let stderr_for_thread = Arc::clone(&stderr_lines);

    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            stderr_for_thread.lock().unwrap().push(line.clone());
            if let Some(parsed) = parse_progress_line(kind, &line) {
                let f = parsed.fraction.clamp(0.0, 1.0);
                let percent = progress_start + (progress_end - progress_start) * f;
                emit_progress_from(&progress_app, percent, &parsed.message);
            }
        }
        let _ = tx_err.send(());
    });

    let stdout_handle = stdout.map(|out| {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            reader
                .lines()
                .map_while(Result::ok)
                .collect::<Vec<_>>()
                .join("\n")
        })
    });

    let status = wait_active_child(&jobs)?;

    let _ = rx_err.recv();

    if jobs.is_cancelled() {
        return Err(CANCELLED_BY_USER.into());
    }

    let stdout_str = if let Some(handle) = stdout_handle {
        handle.join().unwrap_or_default()
    } else {
        String::new()
    };

    if status.success() {
        Ok(stdout_str)
    } else {
        let stderr_str = stderr_lines
            .lock()
            .unwrap()
            .join("\n");
        Err(format_command_failure(
            &stderr_str,
            &stdout_str,
            status.code(),
        ))
    }
}

/// Run a long command without capturing output (legacy fallback).
pub fn run_command_long(program: &Path, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run {}: {e}", program.display()))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Command failed with exit code {:?}",
            status.code()
        ))
    }
}

/// Backwards-compatible alias for short JSON/text commands.
pub fn run_command(program: &Path, args: &[&str], _working_dir: Option<&Path>) -> Result<String, String> {
    run_command_output(program, args)
}

pub fn format_secs(seconds: f64) -> String {
    let total = seconds.max(0.0);
    let h = (total / 3600.0).floor() as u32;
    let m = ((total % 3600.0) / 60.0).floor() as u32;
    let s = total % 60.0;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:06.3}")
    } else {
        format!("{m:02}:{s:06.3}")
    }
}
