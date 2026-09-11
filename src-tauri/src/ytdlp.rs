use crate::ffmpeg;
use crate::models::{YoutubeFormat, YoutubeInfo};
use crate::process_util::{
    find_executable, run_command_output, run_command_output_cancellable, run_command_with_progress,
};
use crate::progress::{PhaseProgress, ProgressKind};
use crate::rate_limit::RateLimiter;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

const PACING_ARGS: &[&str] = &[
    "--sleep-requests",
    "1",
    "--retries",
    "5",
    "--extractor-retries",
    "3",
    "--retry-sleep",
    "exp=1:120",
];
const DOWNLOAD_PACING_ARGS: &[&str] = &[
    "--sleep-interval",
    "1",
    "--max-sleep-interval",
    "5",
    "--concurrent-fragments",
    "1",
    "--limit-rate",
    "8M",
];
const INFO_CACHE_TTL: Duration = Duration::from_secs(3 * 3600);
const OUTDATED_AFTER_DAYS: i64 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DownloadFailureAction {
    Cancelled,
    RateLimited,
    RetryUrl,
    RetryWithoutPrint,
    Fail,
}

enum DownloadSource<'a> {
    Url(&'a str),
    InfoJson(&'a Path),
}

pub fn ytdlp_path() -> Option<PathBuf> {
    find_executable("yt-dlp").or_else(|| find_executable("ytdlp"))
}

pub fn version() -> Option<String> {
    let ytdlp = ytdlp_path()?;
    let output = run_command_output(&ytdlp, &["--version"]).ok()?;
    let line = output.lines().map(str::trim).find(|l| !l.is_empty())?;
    Some(line.to_string())
}

pub fn is_outdated(version: &str, now_epoch_secs: i64) -> bool {
    let Some((year, month, day)) = parse_ytdlp_ymd(version) else {
        return false;
    };
    let version_days = days_from_civil(year, month, day);
    let now_days = now_epoch_secs.div_euclid(86_400);
    now_days.saturating_sub(version_days) >= OUTDATED_AFTER_DAYS
}

pub fn fetch_formats(app: &AppHandle, url: &str) -> Result<YoutubeInfo, String> {
    crate::rate_limit::wait_until_allowed(app)?;
    let ytdlp = ytdlp_path().ok_or("yt-dlp not found on PATH")?;
    let args = build_fetch_args(url);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = match run_command_output_cancellable(app, &ytdlp, &arg_refs) {
        Ok(output) => output,
        Err(e) => return Err(map_ytdlp_error(app, e)),
    };

    let trimmed = output.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        return Err(
            "yt-dlp returned no video data. The URL may be invalid or the video unavailable.".into(),
        );
    }

    let info = parse_youtube_json(&output)?;
    if let Ok(dir) = app_info_cache_dir(app) {
        let _ = write_info_json(&dir, url, &output);
    }
    note_success(app);
    Ok(info)
}

pub fn get_preview_stream_url(url: &str) -> Result<String, String> {
    let ytdlp = ytdlp_path().ok_or("yt-dlp not found on PATH")?;
    let output = run_command_output(
        &ytdlp,
        &[
            "-f",
            "18/22/b[height<=720]/b",
            "--get-url",
            "--no-playlist",
            "--no-warnings",
            url,
        ],
    )?;

    output
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(String::from)
        .ok_or_else(|| "No stream URL returned".to_string())
}

pub fn download_with_format(
    app: &AppHandle,
    url: &str,
    format_id: &str,
    output_path: &str,
    start_secs: Option<f64>,
    end_secs: Option<f64>,
    video_only: bool,
    audio_only: bool,
    convert_to: Option<&str>,
    audio_quality: Option<&str>,
) -> Result<String, String> {
    crate::rate_limit::wait_until_allowed(app)?;
    let ytdlp = ytdlp_path().ok_or("yt-dlp not found on PATH")?;

    let temp_dir = std::env::temp_dir().join("video_trimmer_ytdlp");
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let out_template = temp_dir.join(format!("dl_{stamp}.%(ext)s"));
    let template_str = out_template.to_string_lossy().to_string();

    let format_spec = build_format_spec(format_id, video_only, audio_only);

    let download_phase = PhaseProgress {
        app,
        start: 5.0,
        end: 65.0,
    };
    download_phase.emit_fraction(0.0, "Starting YouTube download…");

    let mut cached_path = app_info_cache_dir(app)
        .ok()
        .and_then(|dir| read_fresh_cache_path(&dir, url, SystemTime::now()));
    let mut used_cache = cached_path.is_some();
    let mut with_print = true;
    let mut last_error = "Download failed".to_string();
    let mut downloaded_path = None;

    for _ in 0..2 {
        let source = if used_cache {
            if let Some(path) = cached_path.as_ref() {
                DownloadSource::InfoJson(path)
            } else {
                DownloadSource::Url(url)
            }
        } else {
            DownloadSource::Url(url)
        };
        let args = build_download_args(&format_spec, &template_str, audio_only, source, with_print);
        match run_download_attempt(app, &ytdlp, &args, &download_phase) {
            Ok(stdout) => {
                note_success(app);
                downloaded_path = Some(resolve_downloaded_path(&stdout, &temp_dir, stamp)?);
                break;
            }
            Err(e) => {
                last_error = e.clone();
                match download_failure_action(&e, used_cache, with_print) {
                    DownloadFailureAction::Cancelled => return Err(e),
                    DownloadFailureAction::RateLimited => return Err(handle_rate_limit(app)),
                    DownloadFailureAction::RetryUrl => {
                        if let Ok(dir) = app_info_cache_dir(app) {
                            evict_info_json(&dir, url);
                        }
                        cached_path = None;
                        used_cache = false;
                    }
                    DownloadFailureAction::RetryWithoutPrint => {
                        with_print = false;
                    }
                    DownloadFailureAction::Fail => return Err(e),
                }
            }
        }
    }

    let downloaded_path = downloaded_path.ok_or(last_error)?;

    download_phase.emit_fraction(1.0, "Download finished");

    if !Path::new(&downloaded_path).exists() {
        return Err(format!(
            "Downloaded file not found at {}",
            downloaded_path
        ));
    }

    let convert_mp3 = convert_to == Some("mp3");
    let final_output = if convert_mp3 {
        ensure_mp3_extension(output_path)
    } else {
        normalize_output_path(output_path, audio_only)
    };

    let needs_trim = match (start_secs, end_secs) {
        (Some(s), Some(e)) if e > s + 0.05 => true,
        _ => false,
    };

    let needs_audio_processing = convert_mp3 || (needs_trim && audio_only);
    let needs_video_processing = needs_trim && !audio_only && !convert_mp3;

    if needs_audio_processing {
        let (s, e) = resolve_audio_range(start_secs, end_secs, &downloaded_path)?;
        let trim_phase = PhaseProgress {
            app,
            start: 68.0,
            end: 92.0,
        };
        ffmpeg::extract_audio(
            app,
            &trim_phase,
            &downloaded_path,
            &final_output,
            s,
            e,
            if convert_mp3 { audio_quality } else { None },
        )?;
        let _ = fs::remove_file(&downloaded_path);
    } else if needs_video_processing {
        let (s, e) = (start_secs.unwrap(), end_secs.unwrap());
        let trim_phase = PhaseProgress {
            app,
            start: 68.0,
            end: 92.0,
        };

        if ffmpeg::trim_video(app, &trim_phase, &downloaded_path, &final_output, s, e, false)
            .is_err()
        {
            ffmpeg::trim_video(app, &trim_phase, &downloaded_path, &final_output, s, e, true)?;
        }
        let _ = fs::remove_file(&downloaded_path);
    } else {
        let save_phase = PhaseProgress {
            app,
            start: 85.0,
            end: 95.0,
        };
        save_phase.emit_fraction(0.0, "Saving file…");
        move_to_output(&downloaded_path, &final_output)?;
        save_phase.emit_fraction(1.0, "Save complete");
    }

    crate::emit_progress_from(app, 98.0, "Verifying output…");
    if !Path::new(&final_output).exists() {
        return Err("Export finished but output file is missing".into());
    }

    Ok(final_output)
}

fn note_success(app: &AppHandle) {
    app.state::<RateLimiter>().record_success(Instant::now());
}

fn handle_rate_limit(app: &AppHandle) -> String {
    let limiter = app.state::<RateLimiter>();
    limiter.record_rate_limit(Instant::now());
    limiter.friendly_message(Instant::now())
}

fn map_ytdlp_error(app: &AppHandle, err: String) -> String {
    if err.contains("Cancelled") {
        return err;
    }
    if crate::rate_limit::classify(&err).is_some() {
        handle_rate_limit(app)
    } else {
        err
    }
}

fn run_download_attempt(
    app: &AppHandle,
    ytdlp: &Path,
    args: &[String],
    download_phase: &PhaseProgress<'_>,
) -> Result<String, String> {
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_command_with_progress(app, ytdlp, &arg_refs, ProgressKind::YtDlp, download_phase)
}

fn download_failure_action(
    err: &str,
    used_cache: bool,
    used_print: bool,
) -> DownloadFailureAction {
    if err.contains("Cancelled") {
        return DownloadFailureAction::Cancelled;
    }
    if crate::rate_limit::classify(err).is_some() {
        return DownloadFailureAction::RateLimited;
    }
    if used_cache {
        return DownloadFailureAction::RetryUrl;
    }
    if used_print && is_print_compat_error(err) {
        return DownloadFailureAction::RetryWithoutPrint;
    }
    DownloadFailureAction::Fail
}

fn is_print_compat_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("--print") || (lower.contains("no such option") && lower.contains("print"))
}

fn extend_args(args: &mut Vec<String>, extra: &[&str]) {
    args.extend(extra.iter().map(|s| (*s).to_string()));
}

fn build_fetch_args(url: &str) -> Vec<String> {
    let mut args = vec![
        "--dump-single-json".into(),
        "--no-playlist".into(),
        "--no-warnings".into(),
    ];
    extend_args(&mut args, PACING_ARGS);
    args.push(url.into());
    args
}

fn build_download_args(
    format_spec: &str,
    template_str: &str,
    audio_only: bool,
    source: DownloadSource<'_>,
    with_print: bool,
) -> Vec<String> {
    let mut args = Vec::new();
    extend_args(&mut args, PACING_ARGS);
    extend_args(&mut args, DOWNLOAD_PACING_ARGS);
    args.extend([
        "-f".into(),
        format_spec.to_string(),
        "--no-playlist".into(),
        "--no-warnings".into(),
        "--newline".into(),
        "--progress".into(),
        "-o".into(),
        template_str.to_string(),
    ]);
    if !audio_only {
        args.push("--merge-output-format".into());
        args.push("mp4".into());
    }
    if with_print {
        args.push("--print".into());
        args.push("after_move:filepath".into());
    }
    match source {
        DownloadSource::Url(source_url) => args.push(source_url.to_string()),
        DownloadSource::InfoJson(path) => {
            args.push("--load-info-json".into());
            args.push(path.to_string_lossy().into_owned());
        }
    }
    args
}

fn cache_key_for_url(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:016x}.json", hasher.finish())
}

fn cache_file_is_fresh(mtime: SystemTime, now: SystemTime, ttl: Duration) -> bool {
    match now.duration_since(mtime) {
        Ok(age) => age <= ttl,
        Err(_) => true,
    }
}

fn write_info_json(cache_dir: &Path, url: &str, json: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(cache_dir).map_err(|e| e.to_string())?;
    let path = cache_dir.join(cache_key_for_url(url));
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

fn read_fresh_cache_path(cache_dir: &Path, url: &str, now: SystemTime) -> Option<PathBuf> {
    let path = cache_dir.join(cache_key_for_url(url));
    let meta = fs::metadata(&path).ok()?;
    let mtime = meta.modified().ok()?;
    cache_file_is_fresh(mtime, now, INFO_CACHE_TTL).then_some(path)
}

fn evict_info_json(cache_dir: &Path, url: &str) {
    let path = cache_dir.join(cache_key_for_url(url));
    let _ = fs::remove_file(path);
}

fn app_info_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Could not resolve app cache: {e}"))?;
    let dir = cache.join("ytdlp-info");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn parse_ytdlp_ymd(version: &str) -> Option<(i32, u32, u32)> {
    for token in version.split_whitespace() {
        let mut parts = token.split('.');
        let Some(year_s) = parts.next() else { continue };
        let Some(month_s) = parts.next() else { continue };
        let Some(day_s) = parts.next() else { continue };
        let Ok(year) = year_s.parse::<i32>() else { continue };
        let Ok(month) = month_s.parse::<u32>() else { continue };
        let Ok(day) = day_s.parse::<u32>() else { continue };
        if year >= 2000 && (1..=12).contains(&month) && (1..=31).contains(&day) {
            return Some((year, month, day));
        }
    }
    None
}

/// Civil date to Unix day count. Howard Hinnant's algorithm; 1970-01-01 => 0.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let mut y = i64::from(year);
    let m = i64::from(month);
    let d = i64::from(day);
    y -= i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = i64::from(yoe) * 365 + i64::from(yoe / 4) - i64::from(yoe / 100) + doy;
    era * 146097 + doe - 719468
}

fn build_format_spec(format_id: &str, video_only: bool, audio_only: bool) -> String {
    if audio_only {
        format_id.to_string()
    } else if video_only {
        // YouTube 1080p+ is usually video-only; merge with the best audio track.
        format!("{format_id}+bestaudio/best")
    } else {
        format_id.to_string()
    }
}

fn resolve_downloaded_path(stdout: &str, temp_dir: &Path, stamp: u128) -> Result<String, String> {
    if let Some(line) = stdout.lines().map(str::trim).filter(|l| !l.is_empty()).last() {
        if Path::new(line).exists() {
            return Ok(line.to_string());
        }
    }
    find_newest_in_dir(temp_dir, stamp)
}

fn find_newest_in_dir(dir: &Path, stamp: u128) -> Result<String, String> {
    let prefix = format!("dl_{stamp}");
    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.starts_with(&prefix) {
            continue;
        }
        if path.is_file() {
            if let Ok(meta) = fs::metadata(&path) {
                if let Ok(modified) = meta.modified() {
                    match &best {
                        None => best = Some((path, modified)),
                        Some((_, t)) if modified > *t => best = Some((path, modified)),
                        _ => {}
                    }
                }
            }
        }
    }

    best.map(|(p, _)| p.to_string_lossy().to_string())
        .ok_or_else(|| "Download completed but the file could not be located".to_string())
}

fn ensure_mp3_extension(output_path: &str) -> String {
    let path = Path::new(output_path);
    match path.extension().and_then(|e| e.to_str()) {
        Some("mp3") => output_path.to_string(),
        _ => {
            let mut p = path.to_path_buf();
            p.set_extension("mp3");
            p.to_string_lossy().to_string()
        }
    }
}

fn resolve_audio_range(
    start_secs: Option<f64>,
    end_secs: Option<f64>,
    input_path: &str,
) -> Result<(f64, f64), String> {
    let probe = ffmpeg::probe_file(input_path)?;
    let file_dur = probe.duration_secs.max(0.1);

    let (start, end) = match (start_secs, end_secs) {
        (Some(s), Some(e)) if e > s + 0.05 => {
            let start = s.clamp(0.0, file_dur);
            let end = e.clamp(start + 0.05, file_dur);
            (start, end)
        }
        _ => (0.0, file_dur),
    };

    Ok((start, end))
}

fn normalize_output_path(output_path: &str, audio_only: bool) -> String {
    let path = Path::new(output_path);
    if audio_only {
        return output_path.to_string();
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("mp4") | Some("mkv") | Some("webm") | Some("mov") => output_path.to_string(),
        _ => {
            let mut p = path.to_path_buf();
            p.set_extension("mp4");
            p.to_string_lossy().to_string()
        }
    }
}

fn move_to_output(from: &str, to: &str) -> Result<(), String> {
    if Path::new(to).exists() {
        fs::remove_file(to).map_err(|e| e.to_string())?;
    }
    if let Some(parent) = Path::new(to).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::rename(from, to).or_else(|_| {
        fs::copy(from, to).map_err(|e| e.to_string())?;
        fs::remove_file(from).map_err(|e| e.to_string())?;
        Ok(())
    })
}

fn parse_youtube_json(json_str: &str) -> Result<YoutubeInfo, String> {
    let json: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid yt-dlp JSON: {e}"))?;

    if json.is_null() {
        return Err("yt-dlp returned no video data.".into());
    }

    let id = json["id"].as_str().unwrap_or("unknown").to_string();
    let title = json["title"].as_str().unwrap_or("Untitled").to_string();
    let duration_secs = json["duration"].as_f64().unwrap_or(0.0);
    let thumbnail = json["thumbnail"].as_str().map(String::from);

    let mut formats = Vec::new();
    if let Some(arr) = json["formats"].as_array() {
        for f in arr {
            let format_id = match f["format_id"].as_str() {
                Some(id) => id.to_string(),
                None => continue,
            };
            let ext = f["ext"].as_str().unwrap_or("unknown").to_string();
            let vcodec = f["vcodec"].as_str().map(String::from);
            let acodec = f["acodec"].as_str().map(String::from);
            let audio_only = vcodec.as_deref() == Some("none");
            let video_only = acodec.as_deref() == Some("none");
            let resolution = f["resolution"].as_str().map(String::from);
            let fps = f["fps"].as_f64();
            let filesize = f["filesize"]
                .as_u64()
                .or_else(|| f["filesize_approx"].as_u64());
            let tbr = f["tbr"].as_f64();
            let format_note = f["format_note"].as_str().map(String::from);

            let label = build_format_label(
                &format_id,
                &ext,
                resolution.as_deref(),
                fps,
                tbr,
                format_note.as_deref(),
                audio_only,
                video_only,
            );

            formats.push(YoutubeFormat {
                format_id,
                ext,
                resolution,
                fps,
                vcodec,
                acodec,
                filesize,
                tbr,
                format_note,
                audio_only,
                video_only,
                label,
            });
        }
    }

    Ok(YoutubeInfo {
        id,
        title,
        duration_secs,
        thumbnail,
        formats,
    })
}

fn build_format_label(
    format_id: &str,
    ext: &str,
    resolution: Option<&str>,
    fps: Option<f64>,
    tbr: Option<f64>,
    note: Option<&str>,
    audio_only: bool,
    video_only: bool,
) -> String {
    let mut parts = vec![format_id.to_string(), ext.to_string()];
    if let Some(res) = resolution {
        if res != "audio only" {
            parts.push(res.to_string());
        }
    }
    if let Some(f) = fps {
        parts.push(format!("{f:.0}fps"));
    }
    if let Some(t) = tbr {
        parts.push(format!("{t:.0}kbps"));
    }
    if audio_only {
        parts.push("audio".into());
    }
    if video_only {
        parts.push("video-only (slower: merges audio)".into());
    }
    if let Some(n) = note {
        parts.push(n.to_string());
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn cache_key_is_stable_and_differs_by_url() {
        let a = cache_key_for_url("https://youtu.be/abc");
        let b = cache_key_for_url("https://youtu.be/abc");
        let c = cache_key_for_url("https://youtu.be/xyz");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.ends_with(".json"));
    }

    #[test]
    fn cache_file_is_fresh_within_ttl_and_stale_after() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000_000);
        let fresh = now - Duration::from_secs(2 * 3600);
        let stale = now - Duration::from_secs(4 * 3600);
        assert!(cache_file_is_fresh(fresh, now, INFO_CACHE_TTL));
        assert!(!cache_file_is_fresh(stale, now, INFO_CACHE_TTL));
        assert!(cache_file_is_fresh(now - INFO_CACHE_TTL, now, INFO_CACHE_TTL));
    }

    #[test]
    fn info_json_cache_roundtrip_and_evict() {
        let dir = std::env::temp_dir().join(format!(
            "vt_info_cache_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let url = "https://youtu.be/cache-roundtrip";
        write_info_json(&dir, url, "{\"id\":\"abc\"}").unwrap();
        let path = read_fresh_cache_path(&dir, url, SystemTime::now()).unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(body, "{\"id\":\"abc\"}");
        evict_info_json(&dir, url);
        assert!(read_fresh_cache_path(&dir, url, SystemTime::now()).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn days_from_civil_unix_epoch_is_zero() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
    }

    #[test]
    fn parses_ytdlp_version_from_output() {
        assert_eq!(parse_ytdlp_ymd("2026.03.17"), Some((2026, 3, 17)));
        assert_eq!(parse_ytdlp_ymd("2026.03.17\n"), Some((2026, 3, 17)));
        assert_eq!(parse_ytdlp_ymd("yt-dlp 2025.12.08"), Some((2025, 12, 8)));
        assert_eq!(parse_ytdlp_ymd("not a version"), None);
    }

    #[test]
    fn is_outdated_after_ninety_days() {
        let release_days = days_from_civil(2026, 3, 17);
        let release_epoch = release_days * 86_400;
        assert!(!is_outdated("2026.03.17", release_epoch + 89 * 86_400));
        assert!(is_outdated("2026.03.17", release_epoch + 90 * 86_400));
        assert!(is_outdated("2026.03.17", release_epoch + 91 * 86_400));
        assert!(!is_outdated("unparseable", release_epoch + 365 * 86_400));
    }

    #[test]
    fn download_failure_prefers_cancel_and_rate_limit() {
        assert_eq!(
            download_failure_action("Cancelled by user", true, true),
            DownloadFailureAction::Cancelled
        );
        assert_eq!(
            download_failure_action("HTTP Error 403: Forbidden", true, true),
            DownloadFailureAction::RateLimited
        );
        assert_eq!(
            download_failure_action("HTTP Error 429: Too Many Requests", false, true),
            DownloadFailureAction::RateLimited
        );
    }

    #[test]
    fn download_failure_retries_cache_then_print_then_fails() {
        assert_eq!(
            download_failure_action("network glitch", true, true),
            DownloadFailureAction::RetryUrl
        );
        assert_eq!(
            download_failure_action("error: no such option --print", false, true),
            DownloadFailureAction::RetryWithoutPrint
        );
        assert_eq!(
            download_failure_action("error: no such option --print", false, false),
            DownloadFailureAction::Fail
        );
        assert_eq!(
            download_failure_action("unable to download video data", false, true),
            DownloadFailureAction::Fail
        );
    }

    #[test]
    fn download_args_include_pacing_and_can_load_info_json() {
        let args = build_download_args(
            "22",
            "C:\\tmp\\dl.%(ext)s",
            false,
            DownloadSource::Url("https://youtu.be/abc"),
            true,
        );
        let joined = args.join(" ");
        assert!(joined.contains("--sleep-requests 1"));
        assert!(joined.contains("--sleep-interval 1"));
        assert!(joined.contains("--concurrent-fragments 1"));
        assert!(joined.contains("--limit-rate 8M"));
        assert!(joined.contains("--print after_move:filepath"));
        assert!(joined.contains("https://youtu.be/abc"));

        let cached = build_download_args(
            "140",
            "C:\\tmp\\dl.%(ext)s",
            true,
            DownloadSource::InfoJson(Path::new("C:\\cache\\info.json")),
            false,
        );
        let cached_joined = cached.join(" ");
        assert!(cached_joined.contains("--load-info-json"));
        assert!(!cached_joined.contains("--print"));
        assert!(!cached_joined.contains("--merge-output-format"));
    }

    #[test]
    fn fetch_args_include_pacing() {
        let args = build_fetch_args("https://youtu.be/abc");
        let joined = args.join(" ");
        assert!(joined.starts_with("--dump-single-json"));
        assert!(joined.contains("--sleep-requests 1"));
        assert!(joined.contains("--retry-sleep exp=1:120"));
        assert!(joined.ends_with("https://youtu.be/abc"));
    }
}
