import { describe, expect, it } from "vitest";
import { getDirectory, sanitizeFilename } from "@/utils/filenames";

describe("sanitizeFilename", () => {
  it("strips reserved characters and collapses whitespace", () => {
    expect(sanitizeFilename('My <Video>: "clip"/one|two?', "fallback")).toBe(
      "My Video clip one two",
    );
  });

  it("returns fallback when nothing usable remains", () => {
    expect(sanitizeFilename("??? ///", "trimmed")).toBe("trimmed");
    expect(sanitizeFilename("   ", "audio")).toBe("audio");
  });

  it("trims trailing dots and spaces and caps length", () => {
    expect(sanitizeFilename("Title...   ", "fallback")).toBe("Title");
    const long = "a".repeat(120);
    expect(sanitizeFilename(long, "fallback").length).toBe(80);
  });
});

describe("getDirectory", () => {
  it("returns the parent of a Windows path", () => {
    expect(getDirectory("C:\\Users\\me\\Downloads\\clip.mp4")).toBe(
      "C:\\Users\\me\\Downloads",
    );
  });

  it("returns the parent of a POSIX path", () => {
    expect(getDirectory("/home/user/videos/clip.mp4")).toBe("/home/user/videos");
  });

  it("returns null when there is no parent", () => {
    expect(getDirectory("clip.mp4")).toBeNull();
  });
});
