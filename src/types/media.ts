export type SourceType = "local" | "youtube";

export interface MediaProbe {
  path: string;
  duration_secs: number;
  width?: number;
  height?: number;
  codec?: string;
  title?: string;
}

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
