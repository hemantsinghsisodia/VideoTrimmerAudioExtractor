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

const mixedFormats: YoutubeFormat[] = [
  ...sampleFormats,
  {
    ...base,
    format_id: "313",
    ext: "webm",
    resolution: "3840x2160",
    audio_only: false,
    video_only: true,
    label: "raw",
    tbr: 15000,
    fps: 30,
  },
  {
    ...base,
    format_id: "sb0",
    ext: "mhtml",
    audio_only: true,
    video_only: true,
    label: "raw",
    tbr: undefined,
    resolution: undefined,
    fps: undefined,
    vcodec: "none",
    acodec: "none",
  },
];

describe("filterFormatsByKind", () => {
  it("filters real audio only", () => {
    const audio = filterFormatsByKind(mixedFormats, "audio");
    expect(audio).toHaveLength(1);
    expect(audio[0].format_id).toBe("140");
  });

  it("filters video and excludes audio and storyboards", () => {
    const video = filterFormatsByKind(mixedFormats, "video");
    expect(video.every((f) => !f.audio_only)).toBe(true);
    expect(video.some((f) => f.ext === "mhtml")).toBe(false);
    expect(video.some((f) => f.format_id === "313")).toBe(true);
  });
});

describe("sortFormatsByQuality", () => {
  it("sorts higher resolution first", () => {
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

  it("lists video options without audio or video-only labels", () => {
    const video = getUserFacingFormats(mixedFormats, "video");
    expect(video.some((f) => f.audio_only)).toBe(false);
    expect(video.some((f) => f.format_id === "313")).toBe(true);
    expect(video.find((f) => f.format_id === "313")?.label).toMatch(/2160p.*WEBM/i);
    expect(video.every((f) => !f.label.includes("video only"))).toBe(true);
    expect(video.find((f) => f.format_id === "137")?.video_only).toBe(true);
  });

  it("uses friendly labels for video and audio tabs", () => {
    const video = getUserFacingFormats(sampleFormats, "video");
    const audio = getUserFacingFormats(sampleFormats, "audio");
    expect(video.find((f) => f.format_id === "18")?.label).toMatch(/360p.*MP4/i);
    expect(audio.find((f) => f.format_id === "140")?.label).toMatch(/Audio.*M4A/i);
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

  it("does not add MP3 options to the video filter", () => {
    const video = getUserFacingFormats(sampleFormats, "video");
    expect(video.some((f) => f.convert_to === "mp3")).toBe(false);
  });

  it("excludes MHTML from the audio list", () => {
    const audio = getUserFacingFormats(mixedFormats, "audio");
    expect(audio.some((f) => f.ext === "mhtml")).toBe(false);
    expect(audio.some((f) => f.format_id === "140")).toBe(true);
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

  it("keeps video-only streams marked so download can merge best audio", () => {
    const video = getUserFacingFormats(mixedFormats, "video");
    const fourK = video.find((f) => f.format_id === "313");
    expect(fourK).toBeDefined();
    expect(resolveYoutubeDownloadFormat(fourK!)).toMatchObject({
      formatId: "313",
      audioOnly: false,
      videoOnly: true,
      defaultExtension: "mp4",
    });
  });
});

describe("pickDefaultFormatId", () => {
  it("defaults to the highest quality video option", () => {
    expect(pickDefaultFormatId(mixedFormats)).toBe("313");
  });
});
