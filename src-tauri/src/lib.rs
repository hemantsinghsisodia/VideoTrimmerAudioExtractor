mod ffmpeg;
mod job_control;
mod models;
mod playback;
mod process_util;
mod progress;
mod ytdlp;

use job_control::JobController;
use models::{DependencyStatus, ExportResult, JobProgress, MediaProbe, YoutubeInfo};
use progress::PhaseProgress;
use tauri::{AppHandle, Emitter, Manager};

pub(crate) fn emit_progress_from(app: &AppHandle, percent: f64, message: &str) {
    let _ = app.emit(
        "job-progress",
        JobProgress {
            job_id: "default".into(),
            percent: percent.clamp(0.0, 100.0),
            message: message.into(),
        },
    );
}

#[tauri::command]
fn cancel_job(jobs: tauri::State<'_, JobController>) -> bool {
    jobs.cancel()
}

#[tauri::command]
fn check_dependencies() -> DependencyStatus {
    let ffmpeg = ffmpeg::ffmpeg_path().is_some();
    let ffprobe = ffmpeg::ffprobe_path().is_some();
    let ytdlp = ytdlp::ytdlp_path().is_some();
    let mut messages = Vec::new();
    if !ffmpeg {
        messages.push("Install ffmpeg and add to PATH".into());
    }
    if !ffprobe {
        messages.push("Install ffprobe (included with ffmpeg)".into());
    }
    if !ytdlp {
        messages.push("Install yt-dlp: pip install yt-dlp".into());
    }
    DependencyStatus {
        ffmpeg,
        ffprobe,
        ytdlp,
        messages,
    }
}

#[tauri::command]
async fn probe_local_file(path: String) -> Result<MediaProbe, String> {
    tauri::async_runtime::spawn_blocking(move || ffmpeg::probe_file(&path))
        .await
        .map_err(|e| format!("Task failed: {e}"))?
}

#[tauri::command]
async fn get_youtube_formats(app: AppHandle, url: String) -> Result<YoutubeInfo, String> {
    let app2 = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app2.state::<JobController>().begin_job();
        emit_progress_from(&app2, 5.0, "Fetching video info…");
        let info = ytdlp::fetch_formats(&app2, &url)?;
        emit_progress_from(&app2, 100.0, "Ready");
        Ok(info)
    })
    .await
    .map_err(|e| format!("Task failed: {e}"))?
}

#[tauri::command]
fn stage_for_playback(app: AppHandle, source_path: String) -> Result<String, String> {
    emit_progress_from(&app, 96.0, "Preparing preview…");
    let path = playback::stage_for_playback(&app, &source_path)?;
    emit_progress_from(&app, 100.0, "Ready");
    Ok(path)
}

#[tauri::command]
async fn get_youtube_preview_url(url: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || ytdlp::get_preview_stream_url(&url))
        .await
        .map_err(|e| format!("Task failed: {e}"))?
}

#[tauri::command]
async fn trim_video(
    app: AppHandle,
    input_path: String,
    output_path: String,
    start_secs: f64,
    end_secs: f64,
    reencode: bool,
) -> Result<ExportResult, String> {
    let app2 = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app2.state::<JobController>().begin_job();
        let phase = PhaseProgress {
            app: &app2,
            start: 5.0,
            end: 95.0,
        };
        ffmpeg::trim_video(
            &app2,
            &phase,
            &input_path,
            &output_path,
            start_secs,
            end_secs,
            reencode,
        )?;
        Ok(ExportResult {
            output_path,
            kind: "trimmed_video".into(),
        })
    })
    .await
    .map_err(|e| format!("Task failed: {e}"))?
}

#[tauri::command]
async fn extract_audio(
    app: AppHandle,
    input_path: String,
    output_path: String,
    start_secs: f64,
    end_secs: f64,
) -> Result<ExportResult, String> {
    let app2 = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app2.state::<JobController>().begin_job();
        let phase = PhaseProgress {
            app: &app2,
            start: 5.0,
            end: 95.0,
        };
        ffmpeg::extract_audio(
            &app2,
            &phase,
            &input_path,
            &output_path,
            start_secs,
            end_secs,
        )?;
        Ok(ExportResult {
            output_path,
            kind: "audio_only".into(),
        })
    })
    .await
    .map_err(|e| format!("Task failed: {e}"))?
}

#[tauri::command]
async fn download_youtube(
    app: AppHandle,
    url: String,
    format_id: String,
    output_path: String,
    start_secs: Option<f64>,
    end_secs: Option<f64>,
    video_only: bool,
    audio_only: bool,
) -> Result<ExportResult, String> {
    let app2 = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app2.state::<JobController>().begin_job();
        emit_progress_from(&app2, 2.0, "Preparing download…");
        let path = ytdlp::download_with_format(
            &app2,
            &url,
            &format_id,
            &output_path,
            start_secs,
            end_secs,
            video_only,
            audio_only,
        )?;
        emit_progress_from(&app2, 100.0, "Complete");
        Ok(ExportResult {
            output_path: path,
            kind: "youtube_format".into(),
        })
    })
    .await
    .map_err(|e| format!("Task failed: {e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(JobController::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_dependencies,
            cancel_job,
            probe_local_file,
            stage_for_playback,
            get_youtube_formats,
            get_youtube_preview_url,
            trim_video,
            extract_audio,
            download_youtube,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
