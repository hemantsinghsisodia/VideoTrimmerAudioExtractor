import { describe, expect, it } from "vitest";
import {
  getFileExtension,
  isAudioOnlyPath,
  isSupportedLocalMediaPath,
  isSupportedVideoPath,
} from "@/utils/videoFiles";

describe("videoFiles", () => {
  it("reads extension from windows paths", () => {
    expect(getFileExtension("C:\\Videos\\clip.mp4")).toBe("mp4");
  });

  it("accepts supported video extensions", () => {
    expect(isSupportedVideoPath("/home/user/movie.mkv")).toBe(true);
    expect(isSupportedLocalMediaPath("/home/user/movie.mkv")).toBe(true);
  });

  it("accepts mp3 as local media", () => {
    expect(isSupportedLocalMediaPath("C:\\Music\\track.mp3")).toBe(true);
    expect(isAudioOnlyPath("C:\\Music\\track.mp3")).toBe(true);
    expect(isSupportedVideoPath("C:\\Music\\track.mp3")).toBe(false);
  });

  it("rejects unsupported extensions", () => {
    expect(isSupportedLocalMediaPath("/home/user/readme.txt")).toBe(false);
    expect(isSupportedVideoPath("/home/user/readme.txt")).toBe(false);
  });
});
