/** Parse HH:MM:SS, MM:SS, or seconds into total seconds */
export function parseTimeInput(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed) return null;

  if (/^\d+(\.\d+)?$/.test(trimmed)) {
    return parseFloat(trimmed);
  }

  const parts = trimmed.split(":").map((p) => p.trim());
  if (parts.some((p) => p === "" || Number.isNaN(Number(p)))) {
    return null;
  }

  const nums = parts.map(Number);
  if (nums.some((n) => n < 0)) return null;

  if (parts.length === 2) {
    const [mm, ss] = nums;
    if (ss >= 60) return null;
    return mm * 60 + ss;
  }

  if (parts.length === 3) {
    const [hh, mm, ss] = nums;
    if (mm >= 60 || ss >= 60) return null;
    return hh * 3600 + mm * 60 + ss;
  }

  return null;
}

export function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "00:00";
  const total = Math.floor(seconds);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  if (h > 0) return `${pad(h)}:${pad(m)}:${pad(s)}`;
  return `${pad(m)}:${pad(s)}`;
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export interface TrimValidationResult {
  valid: boolean;
  start: number;
  end: number;
  error?: string;
}

export function validateTrimRange(
  start: number,
  end: number,
  duration: number,
): TrimValidationResult {
  if (!Number.isFinite(duration) || duration <= 0) {
    return { valid: false, start: 0, end: 0, error: "Invalid video duration" };
  }
  if (!Number.isFinite(start) || !Number.isFinite(end)) {
    return { valid: false, start: 0, end: 0, error: "Start and end must be numbers" };
  }
  if (start < 0 || end < 0) {
    return { valid: false, start, end, error: "Times cannot be negative" };
  }
  if (start >= end) {
    return { valid: false, start, end, error: "Start must be before end" };
  }
  if (end > duration) {
    return {
      valid: false,
      start,
      end,
      error: `End time cannot exceed duration (${formatTime(duration)})`,
    };
  }
  return { valid: true, start, end };
}
