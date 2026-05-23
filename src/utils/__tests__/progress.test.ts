import { describe, expect, it } from "vitest";
import { isCancelledError } from "@/utils/progress";

describe("isCancelledError", () => {
  it("detects cancelled messages", () => {
    expect(isCancelledError(new Error("Cancelled by user"))).toBe(true);
    expect(isCancelledError("failed")).toBe(false);
  });
});
