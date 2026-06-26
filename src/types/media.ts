export type SourceType = "local" | "youtube";

export interface MediaProbe {
  path: string;
  duration_secs: number;
  width?: number;
  height?: number;
  codec?: string;
  title?: string;
}

export type AudioConvertTarget = "mp3";

export interface YoutubeFormat {
  format_id: string;
  ext: string;
  resolution?: string;
  fps?: number;
  vcodec?: string;
  acodec?: string;
  filesize?: number;
  tbr?: number;
  format_note?: string;
  audio_only: boolean;
  video_only: boolean;
  label: string;
  /** When set, export downloads `source_format_id` and converts to this container. */
  convert_to?: AudioConvertTarget;
  /** yt-dlp format id used as the download source for converted audio options. */
  source_format_id?: string;
  /** MP3 encoder preset, e.g. `320` (CBR) or `v0` (highest VBR). */
  audio_quality?: string;
}

export interface YoutubeInfo {
  id: string;
  title: string;
  duration_secs: number;
  thumbnail?: string;
  formats: YoutubeFormat[];
}

export interface TrimRange {
  start: number;
  end: number;
}

export type ExportKind = "trimmed_video" | "audio_only" | "youtube_format";

export interface JobProgress {
  job_id: string;
  percent: number;
  message: string;
}

export interface ExportResult {
  output_path: string;
  kind: ExportKind;
}
