use std::path::Path;
use tauri::{AppHandle, Manager};

pub fn stage_for_playback(app: &AppHandle, source_path: &str) -> Result<String, String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(format!("File not found: {source_path}"));
    }

    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Could not resolve app cache: {e}"))?;

    let playback_dir = cache.join("playback");
    std::fs::create_dir_all(&playback_dir).map_err(|e| e.to_string())?;

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4");

    let dest = playback_dir.join(format!("current.{ext}"));
    if dest.exists() {
        std::fs::remove_file(&dest).ok();
    }

    std::fs::copy(source, &dest).map_err(|e| format!("Failed to stage file for playback: {e}"))?;

    Ok(dest.to_string_lossy().to_string())
}
