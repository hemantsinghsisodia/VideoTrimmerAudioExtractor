import type { YoutubeFormat } from "@/types/media";

export type FormatFilterKind = "all" | "video" | "audio";

export function filterFormatsByKind(
  formats: YoutubeFormat[],
  kind: FormatFilterKind,
): YoutubeFormat[] {
  switch (kind) {
    case "video":
      return formats.filter((f) => !f.audio_only);
    case "audio":
      return formats.filter((f) => f.audio_only);
    default:
      return formats;
  }
}

export function sortFormatsByQuality(formats: YoutubeFormat[]): YoutubeFormat[] {
  return [...formats].sort((a, b) => qualityScore(b) - qualityScore(a));
}

/** Dedupe raw yt-dlp formats and return friendly labels for the UI. */
export function getUserFacingFormats(
  formats: YoutubeFormat[],
  kind: FormatFilterKind,
): YoutubeFormat[] {
  const filtered = filterFormatsByKind(formats, kind);

  if (kind === "all") {
    const combined = dedupeByGroup(filtered.filter((f) => !f.audio_only && !f.video_only));
    const audio = dedupeByGroup(filtered.filter((f) => f.audio_only)).slice(0, 3);
    return sortFormatsByQuality([
      ...combined.map(withFriendlyLabel),
      ...audio.map(withFriendlyLabel),
    ]);
  }

  if (kind === "video") {
    const combined = dedupeByGroup(filtered.filter((f) => !f.video_only));
    const videoOnly = dedupeByGroup(filtered.filter((f) => f.video_only));
    return sortFormatsByQuality([
      ...combined.map(withFriendlyLabel),
      ...videoOnly.map(withFriendlyLabel),
    ]);
  }

  return sortFormatsByQuality(
    dedupeByGroup(filtered).map(withFriendlyLabel),
  );
}

export function pickDefaultFormatId(formats: YoutubeFormat[]): string | null {
  const recommended = getUserFacingFormats(formats, "all");
  const pick =
    recommended.find((f) => f.format_id === "18") ??
    recommended.find((f) => f.format_id === "22") ??
    recommended.find((f) => !f.audio_only && !f.video_only) ??
    recommended[0];
  return pick?.format_id ?? null;
}

function dedupeByGroup(formats: YoutubeFormat[]): YoutubeFormat[] {
  const map = new Map<string, YoutubeFormat>();
  for (const f of formats) {
    const key = groupKey(f);
    const existing = map.get(key);
    if (!existing || qualityScore(f) > qualityScore(existing)) {
      map.set(key, f);
    }
  }
  return Array.from(map.values());
}

function groupKey(f: YoutubeFormat): string {
  if (f.audio_only) {
    return `audio:${f.ext}`;
  }
  const height = getHeight(f);
  const fps = f.fps ? Math.round(f.fps) : 0;
  const kind = f.video_only ? "video_only" : "combined";
  return `${kind}:${f.ext}:${height}:${fps}`;
}

function getHeight(f: YoutubeFormat): number {
  if (!f.resolution) return 0;
  const h = parseInt(f.resolution.split("x")[1] ?? "0", 10);
  return Number.isFinite(h) ? h : 0;
}

function withFriendlyLabel(f: YoutubeFormat): YoutubeFormat {
  return { ...f, label: buildFriendlyLabel(f) };
}

function buildFriendlyLabel(f: YoutubeFormat): string {
  if (f.audio_only) {
    const kbps = f.tbr ? `${Math.round(f.tbr)} kbps` : "";
    const parts = ["Audio", f.ext.toUpperCase()];
    if (kbps) parts.push(kbps);
    return parts.join(" · ");
  }

  const height = getHeight(f);
  const resLabel = height > 0 ? `${height}p` : "Video";
  const fps =
    f.fps && f.fps >= 50 ? `${Math.round(f.fps)}fps` : f.fps ? `${Math.round(f.fps)}fps` : "";
  const quality = fps && !resLabel.includes("fps") ? `${resLabel}${fps}` : resLabel;
  const ext = f.ext.toUpperCase();

  if (f.video_only) {
    return `${quality} ${ext} (video only)`;
  }
  return `${quality} ${ext}`;
}

function qualityScore(f: YoutubeFormat): number {
  let score = 0;
  if (!f.video_only && !f.audio_only) score += 10_000;
  score += getHeight(f) * 10;
  if (f.fps) score += f.fps;
  if (f.tbr) score += f.tbr;
  if (f.filesize) score += f.filesize / 1_000_000;
  return score;
}
