import { describe, expect, it } from "vitest";
import type { YoutubeFormat } from "@/types/media";
import {
  filterFormatsByKind,
  getUserFacingFormats,
  pickDefaultFormatId,
  resolveYoutubeDownloadFormat,
  sortFormatsByQuality,
} from "@/utils/formats";

const base = {
  vcodec: "avc1",
  acodec: "mp4a",
  filesize: undefined,
  format_note: undefined,
} satisfies Partial<YoutubeFormat>;

const sampleFormats: YoutubeFormat[] = [
  {
    ...base,
    format_id: "140",
    ext: "m4a",
    audio_only: true,
    video_only: false,
    label: "raw",
    tbr: 128,
    resolution: undefined,
    fps: undefined,
  },
  {
    ...base,
    format_id: "137",
    ext: "mp4",
    resolution: "1920x1080",
    audio_only: false,
    video_only: true,
    label: "raw",
    tbr: 5000,
    fps: 30,
  },
  {
    ...base,
    format_id: "18",
    ext: "mp4",
    resolution: "640x360",
    audio_only: false,
    video_only: false,
    label: "raw",
    tbr: 500,
    fps: 30,
  },
];

const duplicate300Formats: YoutubeFormat[] = [
  {
    ...base,
    format_id: "300-21",
    ext: "mp4",
    resolution: "1280x720",
    audio_only: false,
    video_only: true,
    label: "raw",
    tbr: 4049,
    fps: 60,
    format_note: "original",
  },
  {
    ...base,
    format_id: "300-0",
    ext: "mp4",
    resolution: "1280x720",
    audio_only: false,
    video_only: true,
    label: "raw",
    tbr: 4049,
    fps: 60,
  },
  {
    ...base,
    format_id: "300-5",
    ext: "mp4",
    resolution: "1280x720",
    audio_only: false,
    video_only: true,
    label: "raw",
    tbr: 4049,
    fps: 60,
  },
  {
    ...base,
    format_id: "18",
    ext: "mp4",
    resolution: "640x360",
    audio_only: false,
    video_only: false,
    label: "raw",
    tbr: 544,
    fps: 30,
  },
];

describe("filterFormatsByKind", () => {
  it("filters audio only", () => {
    const audio = filterFormatsByKind(sampleFormats, "audio");
    expect(audio).toHaveLength(1);
    expect(audio[0].format_id).toBe("140");
  });

  it("filters video (non audio-only)", () => {
    const video = filterFormatsByKind(sampleFormats, "video");
    expect(video.every((f) => !f.audio_only)).toBe(true);
  });
});

describe("sortFormatsByQuality", () => {
  it("sorts higher quality first", () => {
    const sorted = sortFormatsByQuality(sampleFormats);
    expect(sorted[0].format_id).toBe("137");
  });
});

describe("getUserFacingFormats", () => {
  it("collapses duplicate 300-* variants into one video option", () => {
    const video = getUserFacingFormats(duplicate300Formats, "video");
    const sevenTwenty = video.filter((f) => f.label.includes("720"));
    expect(sevenTwenty).toHaveLength(1);
    expect(sevenTwenty[0].format_id).toBe("300-21");
    expect(sevenTwenty[0].label).toContain("720p");
  });

  it("includes combined and audio in recommended list", () => {
    const all = getUserFacingFormats(
      [...duplicate300Formats, ...sampleFormats.filter((f) => f.format_id === "140")],
      "all",
    );
    expect(all.some((f) => f.format_id === "18")).toBe(true);
    expect(all.some((f) => f.audio_only)).toBe(true);
    expect(all.filter((f) => f.label.includes("720")).length).toBeLessThanOrEqual(1);
  });

  it("uses friendly labels instead of raw format ids", () => {
    const all = getUserFacingFormats(sampleFormats, "all");
    expect(all.find((f) => f.format_id === "18")?.label).toMatch(/360p.*MP4/i);
    expect(all.find((f) => f.format_id === "140")?.label).toMatch(/Audio.*M4A/i);
  });

  it("adds high-quality MP3 options to the audio filter", () => {
    const audioFormats = getUserFacingFormats(
      [
        ...sampleFormats,
        {
          ...base,
          format_id: "251",
          ext: "webm",
          audio_only: true,
          video_only: false,
          label: "raw",
          tbr: 160,
          resolution: undefined,
          fps: undefined,
        },
      ],
      "audio",
    );

    const mp3Options = audioFormats.filter((f) => f.convert_to === "mp3");
    expect(mp3Options.length).toBeGreaterThanOrEqual(2);
    expect(mp3Options.some((f) => f.label.includes("MP3") && f.label.includes("320"))).toBe(true);
    expect(mp3Options.some((f) => f.label.includes("MP3") && f.label.includes("V0"))).toBe(true);
    expect(mp3Options.every((f) => f.source_format_id === "140")).toBe(true);
    expect(mp3Options.every((f) => f.ext === "mp3")).toBe(true);
  });

  it("does not add MP3 options to video or recommended filters", () => {
    const video = getUserFacingFormats(sampleFormats, "video");
    const all = getUserFacingFormats(sampleFormats, "all");
    expect(video.some((f) => f.convert_to === "mp3")).toBe(false);
    expect(all.some((f) => f.convert_to === "mp3")).toBe(false);
  });
});

describe("resolveYoutubeDownloadFormat", () => {
  it("maps MP3 conversion options to source format and mp3 extension", () => {
    const mp3Option: YoutubeFormat = {
      ...base,
      format_id: "mp3-320@140",
      ext: "mp3",
      audio_only: true,
      video_only: false,
      label: "Audio · MP3 · 320 kbps",
      convert_to: "mp3",
      source_format_id: "140",
      audio_quality: "320",
      tbr: 128,
      resolution: undefined,
      fps: undefined,
    };

    expect(resolveYoutubeDownloadFormat(mp3Option)).toEqual({
      formatId: "140",
      audioOnly: true,
      videoOnly: false,
      convertTo: "mp3",
      audioQuality: "320",
      defaultExtension: "mp3",
    });
  });
});

describe("pickDefaultFormatId", () => {
  it("prefers combined format 18", () => {
    expect(pickDefaultFormatId(duplicate300Formats)).toBe("18");
  });
});
