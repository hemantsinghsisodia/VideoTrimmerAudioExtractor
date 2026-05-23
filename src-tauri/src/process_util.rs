use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

/// Run a long command (download/encode). Streams stderr to the terminal; no pipe deadlock.
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
