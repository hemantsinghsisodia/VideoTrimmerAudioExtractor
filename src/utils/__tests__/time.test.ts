import { describe, expect, it } from "vitest";
import { formatTime, parseTimeInput, validateTrimRange } from "@/utils/time";

describe("parseTimeInput", () => {
  it("parses seconds", () => {
    expect(parseTimeInput("90")).toBe(90);
    expect(parseTimeInput("12.5")).toBe(12.5);
  });

  it("parses MM:SS", () => {
    expect(parseTimeInput("01:30")).toBe(90);
    expect(parseTimeInput("0:05")).toBe(5);
  });

  it("parses HH:MM:SS", () => {
    expect(parseTimeInput("1:01:01")).toBe(3661);
  });

  it("rejects invalid input", () => {
    expect(parseTimeInput("")).toBeNull();
    expect(parseTimeInput("abc")).toBeNull();
    expect(parseTimeInput("01:99")).toBeNull();
  });
});

describe("formatTime", () => {
  it("formats under one hour", () => {
    expect(formatTime(65)).toBe("01:05");
  });

  it("formats with hours", () => {
    expect(formatTime(3661)).toBe("01:01:01");
  });
});

describe("validateTrimRange", () => {
  it("accepts valid range", () => {
    const r = validateTrimRange(10, 60, 120);
    expect(r.valid).toBe(true);
    expect(r.start).toBe(10);
    expect(r.end).toBe(60);
  });

  it("rejects start >= end", () => {
    const r = validateTrimRange(60, 10, 120);
    expect(r.valid).toBe(false);
  });

  it("rejects end beyond duration", () => {
    const r = validateTrimRange(0, 150, 120);
    expect(r.valid).toBe(false);
    expect(r.error).toContain("duration");
  });
});
