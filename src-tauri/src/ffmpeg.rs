use crate::models::MediaProbe;
use crate::process_util::{find_executable, format_secs, run_command_long, run_command_output};
use std::path::{Path, PathBuf};

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
    input: &str,
    output: &str,
    start_secs: f64,
    end_secs: f64,
    reencode: bool,
) -> Result<(), String> {
    let ffmpeg = ffmpeg_path().ok_or("ffmpeg not found on PATH")?;
    let duration = (end_secs - start_secs).max(0.1);
    let start = format_secs(start_secs);

    let mut args: Vec<String> = vec![
        "-y".into(),
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
        args.extend(["-c".into(), "copy".into(), "-avoid_negative_ts".into(), "make_zero".into()]);
    }

    args.push(output.into());

    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_command_long(&ffmpeg, &arg_refs)?;

    if !reencode && output.ends_with(".mp4") {
        let _ = optimize_for_player(&ffmpeg, output);
    }

    Ok(())
}

/// Move MP4 metadata to the start so HTML5 video can play before full download.
fn optimize_for_player(ffmpeg: &Path, output: &str) -> Result<(), String> {
    let temp = format!("{output}.faststart.mp4");
    let status = run_command_long(
        ffmpeg,
        &[
            "-y",
            "-i",
            output,
            "-c",
            "copy",
            "-movflags",
            "+faststart",
            &temp,
        ],
    );
    if status.is_ok() && Path::new(&temp).exists() {
        std::fs::remove_file(output).ok();
        std::fs::rename(&temp, output).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn extract_audio(
    input: &str,
    output: &str,
    start_secs: f64,
    end_secs: f64,
) -> Result<(), String> {
    let ffmpeg = ffmpeg_path().ok_or("ffmpeg not found on PATH")?;
    let duration = (end_secs - start_secs).max(0.1);
    let start = format_secs(start_secs);

    let ext = Path::new(output)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("m4a");

    let (audio_codec, audio_args): (&str, Vec<&str>) = match ext.to_lowercase().as_str() {
        "mp3" => ("libmp3lame", vec!["-q:a", "0"]),
        "wav" => ("pcm_s16le", vec![]),
        "flac" => ("flac", vec![]),
        _ => ("aac", vec!["-b:a", "320k"]),
    };

    let duration_str = format!("{duration:.3}");
    let mut args = vec![
        "-y",
        "-ss",
        &start,
        "-i",
        input,
        "-t",
        duration_str.as_str(),
        "-vn",
        "-c:a",
        audio_codec,
    ];
    for a in &audio_args {
        args.push(a);
    }
    args.push(output);

    run_command_long(&ffmpeg, &args)?;
    Ok(())
}
