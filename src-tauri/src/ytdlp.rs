use crate::ffmpeg;
use crate::models::{YoutubeFormat, YoutubeInfo};
use crate::process_util::{
    find_executable, run_command_output, run_command_output_cancellable, run_command_with_progress,
};
use crate::progress::{PhaseProgress, ProgressKind};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

pub fn ytdlp_path() -> Option<PathBuf> {
    find_executable("yt-dlp").or_else(|| find_executable("ytdlp"))
}

pub fn fetch_formats(app: &AppHandle, url: &str) -> Result<YoutubeInfo, String> {
    let ytdlp = ytdlp_path().ok_or("yt-dlp not found on PATH")?;
    let output = run_command_output_cancellable(
        app,
        &ytdlp,
        &["--dump-single-json", "--no-playlist", "--no-warnings", url],
    )?;

    let trimmed = output.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        return Err(
            "yt-dlp returned no video data. The URL may be invalid or the video unavailable.".into(),
        );
    }

    parse_youtube_json(&output)
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

    let mut args: Vec<String> = vec![
        "-f".into(),
        format_spec,
        "--no-playlist".into(),
        "--no-warnings".into(),
        "--newline".into(),
        "--progress".into(),
        "-o".into(),
        template_str.clone(),
        url.into(),
    ];

    if !audio_only {
        args.push("--merge-output-format".into());
        args.push("mp4".into());
    }

    args.push("--print".into());
    args.push("after_move:filepath".into());

    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    let downloaded_path = match run_command_with_progress(
        app,
        &ytdlp,
        &arg_refs,
        ProgressKind::YtDlp,
        &download_phase,
    ) {
        Ok(stdout) => resolve_downloaded_path(&stdout, &temp_dir, stamp)?,
        Err(e) if e.contains("Cancelled") => return Err(e),
        Err(_) => {
            args.pop();
            args.pop();
            let arg_refs2: Vec<&str> = args.iter().map(String::as_str).collect();
            run_command_with_progress(app, &ytdlp, &arg_refs2, ProgressKind::YtDlp, &download_phase)?;
            find_newest_in_dir(&temp_dir, stamp)?
        }
    };

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

fn build_format_spec(format_id: &str, video_only: bool, audio_only: bool) -> String {
    if audio_only {
        format_id.to_string()
    } else if video_only {
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
