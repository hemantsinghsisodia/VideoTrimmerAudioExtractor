use crate::models::MediaProbe;
use crate::process_util::{
    find_executable, format_secs, run_command_output, run_command_with_progress,
};
use crate::progress::{PhaseProgress, ProgressKind};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

pub fn ffmpeg_path() -> Option<PathBuf> {
    find_executable("ffmpeg")
}

pub fn ffprobe_path() -> Option<PathBuf> {
    find_executable("ffprobe")
}

pub fn probe_file(path: &str) -> Result<MediaProbe, String> {
    let ffprobe = ffprobe_path().ok_or("ffprobe not found on PATH")?;
    let output = run_command_output(
        &ffprobe,
        &[
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path,
        ],
    )?;

    let json: serde_json::Value =
        serde_json::from_str(&output).map_err(|e| format!("Invalid ffprobe JSON: {e}"))?;

    let duration_secs = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let mut width = None;
    let mut height = None;
    let mut codec = None;

    if let Some(streams) = json["streams"].as_array() {
        for stream in streams {
            if stream["codec_type"].as_str() == Some("video") {
                width = stream["width"].as_u64().map(|v| v as u32);
                height = stream["height"].as_u64().map(|v| v as u32);
                codec = stream["codec_name"].as_str().map(String::from);
                break;
            }
        }
    }

    let title = json["format"]["tags"]["title"]
        .as_str()
        .map(String::from)
        .or_else(|| Path::new(path).file_name().and_then(|n| n.to_str().map(String::from)));

    Ok(MediaProbe {
        path: path.to_string(),
        duration_secs,
        width,
        height,
        codec,
        title,
    })
}

pub fn trim_video(
    app: &AppHandle,
    progress: &PhaseProgress<'_>,
    input: &str,
    output: &str,
    start_secs: f64,
    end_secs: f64,
    reencode: bool,
) -> Result<(), String> {
    let ffmpeg = ffmpeg_path().ok_or("ffmpeg not found on PATH")?;
    let duration = (end_secs - start_secs).max(0.1);
    let start = format_secs(start_secs);

    progress.emit_fraction(0.0, "Starting trim…");

    let mut args: Vec<String> = vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "info".into(),
        "-ss".into(),
        start,
        "-i".into(),
        input.into(),
        "-t".into(),
        format!("{duration:.3}"),
    ];

    if reencode {
        args.extend([
            "-c:v".into(),
            "libx264".into(),
            "-preset".into(),
            "fast".into(),
            "-crf".into(),
            "23".into(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "192k".into(),
            "-movflags".into(),
            "+faststart".into(),
        ]);
    } else {
        args.extend([
            "-c".into(),
            "copy".into(),
            "-avoid_negative_ts".into(),
            "make_zero".into(),
        ]);
    }

    args.push(output.into());

    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_command_with_progress(
        app,
        &ffmpeg,
        &arg_refs,
        ProgressKind::Ffmpeg { duration_secs: duration },
        progress,
    )?;

    if !reencode && output.ends_with(".mp4") {
        progress.emit_fraction(0.95, "Optimizing for playback…");
        let _ = optimize_for_player(app, progress, &ffmpeg, output);
    }

    progress.emit_fraction(1.0, "Trim complete");
    Ok(())
}

fn optimize_for_player(
    app: &AppHandle,
    progress: &PhaseProgress<'_>,
    ffmpeg: &Path,
    output: &str,
) -> Result<(), String> {
    let temp = format!("{output}.faststart.mp4");
    let phase = PhaseProgress {
        app,
        start: progress.start,
        end: progress.end,
    };
    let _ = run_command_with_progress(
        app,
        ffmpeg,
        &[
            "-y",
            "-hide_banner",
            "-loglevel",
            "info",
            "-i",
            output,
            "-c",
            "copy",
            "-movflags",
            "+faststart",
            &temp,
        ],
        ProgressKind::Ffmpeg {
            duration_secs: 1.0,
        },
        &phase,
    );
    if Path::new(&temp).exists() {
        std::fs::remove_file(output).ok();
        std::fs::rename(&temp, output).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn extract_audio(
    app: &AppHandle,
    progress: &PhaseProgress<'_>,
    input: &str,
    output: &str,
    start_secs: f64,
    end_secs: f64,
    mp3_quality: Option<&str>,
) -> Result<(), String> {
    let ffmpeg = ffmpeg_path().ok_or("ffmpeg not found on PATH")?;
    let duration = (end_secs - start_secs).max(0.1);
    let start = format_secs(start_secs);

    if let Some(parent) = Path::new(output).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    progress.emit_fraction(0.0, "Extracting audio…");

    let ext = Path::new(output)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("m4a");

    let mut audio_args: Vec<&str> = Vec::new();
    let audio_codec = match ext.to_lowercase().as_str() {
        "mp3" => {
            audio_args.extend(["-ar", "44100", "-ac", "2"]);
            match mp3_quality {
                Some("320") => {
                    audio_args.extend(["-b:a", "320k"]);
                    "libmp3lame"
                }
                _ => {
                    audio_args.extend(["-q:a", "0"]);
                    "libmp3lame"
                }
            }
        }
        "wav" => "pcm_s16le",
        "flac" => "flac",
        _ => {
            audio_args.extend(["-b:a", "320k"]);
            "aac"
        }
    };

    let duration_str = format!("{duration:.3}");
    let mut args = vec![
        "-y",
        "-hide_banner",
        "-loglevel",
        "warning",
        "-i",
        input,
        "-ss",
        &start,
        "-t",
        duration_str.as_str(),
        "-vn",
        "-map",
        "0:a:0?",
        "-c:a",
        audio_codec,
    ];
    args.extend(audio_args);
    if ext.eq_ignore_ascii_case("mp3") {
        args.push("-f");
        args.push("mp3");
    }
    args.push(output);

    run_command_with_progress(
        app,
        &ffmpeg,
        &args,
        ProgressKind::Ffmpeg { duration_secs: duration },
        progress,
    )?;

    progress.emit_fraction(1.0, "Audio extraction complete");
    Ok(())
}
