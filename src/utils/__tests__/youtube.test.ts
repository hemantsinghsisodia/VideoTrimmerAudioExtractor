import { describe, expect, it } from "vitest";
import { extractYoutubeId, validateYoutubeUrl } from "@/utils/youtube";

describe("validateYoutubeUrl", () => {
  it("accepts standard watch URLs", () => {
    expect(
      validateYoutubeUrl("https://www.youtube.com/watch?v=dQw4w9WgXcQ").valid,
    ).toBe(true);
  });

  it("accepts youtu.be links", () => {
    expect(validateYoutubeUrl("https://youtu.be/dQw4w9WgXcQ").valid).toBe(true);
  });

  it("rejects non-YouTube URLs", () => {
    expect(validateYoutubeUrl("https://example.com/video").valid).toBe(false);
  });
});

describe("extractYoutubeId", () => {
  it("extracts from watch URL", () => {
    expect(extractYoutubeId("https://youtube.com/watch?v=abc123XYZ-_")).toBe(
      "abc123XYZ-_",
    );
  });

  it("extracts from shorts", () => {
    expect(extractYoutubeId("https://youtube.com/shorts/abc123")).toBe("abc123");
  });
});
