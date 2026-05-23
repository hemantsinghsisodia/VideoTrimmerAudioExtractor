import { describe, expect, it } from "vitest";
import { getFileExtension, isSupportedVideoPath } from "@/utils/videoFiles";

describe("videoFiles", () => {
  it("reads extension from windows paths", () => {
    expect(getFileExtension("C:\\Videos\\clip.mp4")).toBe("mp4");
  });

  it("accepts supported video extensions", () => {
    expect(isSupportedVideoPath("/home/user/movie.mkv")).toBe(true);
  });

  it("rejects unsupported extensions", () => {
    expect(isSupportedVideoPath("/home/user/readme.txt")).toBe(false);
  });
});
