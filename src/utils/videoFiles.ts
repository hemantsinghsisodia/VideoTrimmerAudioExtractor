export const VIDEO_FILE_EXTENSIONS = [
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
] as const;

export const AUDIO_FILE_EXTENSIONS = ["mp3"] as const;

export const LOCAL_MEDIA_FILE_EXTENSIONS = [
  ...VIDEO_FILE_EXTENSIONS,
  ...AUDIO_FILE_EXTENSIONS,
] as const;

export function getFileExtension(path: string): string | null {
  const name = path.replace(/\\/g, "/").split("/").pop() ?? "";
  const dot = name.lastIndexOf(".");
  if (dot <= 0 || dot === name.length - 1) return null;
  return name.slice(dot + 1).toLowerCase();
}

export function isSupportedVideoPath(path: string): boolean {
  const ext = getFileExtension(path);
  if (!ext) return false;
  return (VIDEO_FILE_EXTENSIONS as readonly string[]).includes(ext);
}

export function isSupportedLocalMediaPath(path: string): boolean {
  const ext = getFileExtension(path);
  if (!ext) return false;
  return (LOCAL_MEDIA_FILE_EXTENSIONS as readonly string[]).includes(ext);
}

export function isAudioOnlyPath(path: string): boolean {
  const ext = getFileExtension(path);
  if (!ext) return false;
  return (AUDIO_FILE_EXTENSIONS as readonly string[]).includes(ext);
}
