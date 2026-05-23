import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  ExportKind,
  ExportResult,
  JobProgress,
  MediaProbe,
  YoutubeInfo,
} from "@/types/media";

export async function pickVideoFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "Video",
        extensions: [
          "mp4",
          "mkv",
          "webm",
          "avi",
          "mov",
          "flv",
          "wmv",
          "m4v",
          "mpeg",
          "mpg",
        ],
      },
    ],
  });
  if (!selected || Array.isArray(selected)) return null;
  return selected;
}

export async function pickSavePath(defaultName: string): Promise<string | null> {
  const path = await save({
    defaultPath: defaultName,
  });
  return path;
}

export async function probeLocalFile(path: string): Promise<MediaProbe> {
  return invoke<MediaProbe>("probe_local_file", { path });
}

export async function getYoutubeFormats(url: string): Promise<YoutubeInfo> {
  return invoke<YoutubeInfo>("get_youtube_formats", { url });
}

export async function getYoutubePreviewUrl(url: string): Promise<string> {
  return invoke<string>("get_youtube_preview_url", { url });
}

export async function trimVideo(
  inputPath: string,
  outputPath: string,
  startSecs: number,
  endSecs: number,
  reencode: boolean,
): Promise<ExportResult> {
  return invoke<ExportResult>("trim_video", {
    inputPath,
    outputPath,
    startSecs,
    endSecs,
    reencode,
  });
}

export async function extractAudio(
  inputPath: string,
  outputPath: string,
  startSecs: number,
  endSecs: number,
): Promise<ExportResult> {
  return invoke<ExportResult>("extract_audio", {
    inputPath,
    outputPath,
    startSecs,
    endSecs,
  });
}

export interface YoutubeDownloadParams {
  url: string;
  formatId: string;
  outputPath: string;
  startSecs?: number;
  endSecs?: number;
  videoOnly: boolean;
  audioOnly: boolean;
}

export async function downloadYoutube(params: YoutubeDownloadParams): Promise<ExportResult> {
  return invoke<ExportResult>("download_youtube", {
    url: params.url,
    formatId: params.formatId,
    outputPath: params.outputPath,
    startSecs: params.startSecs,
    endSecs: params.endSecs,
    videoOnly: params.videoOnly,
    audioOnly: params.audioOnly,
  });
}

export async function checkDependencies(): Promise<{
  ffmpeg: boolean;
  ffprobe: boolean;
  ytdlp: boolean;
  messages: string[];
}> {
  return invoke("check_dependencies");
}

export function onJobProgress(callback: (progress: JobProgress) => void): Promise<UnlistenFn> {
  return listen<JobProgress>("job-progress", (event) => {
    callback(event.payload);
  });
}

export function videoSrcFromPath(path: string): string {
  return convertFileSrc(path, "asset");
}

export async function stageForPlayback(sourcePath: string): Promise<string> {
  return invoke<string>("stage_for_playback", { sourcePath });
}

export type { ExportKind };
