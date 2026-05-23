import { z } from "zod";

const youtubeUrlSchema = z
  .string()
  .trim()
  .url()
  .refine(
    (url) => {
      try {
        const u = new URL(url);
        const host = u.hostname.replace(/^www\./, "");
        return (
          host === "youtube.com" ||
          host === "youtu.be" ||
          host === "m.youtube.com" ||
          host === "music.youtube.com"
        );
      } catch {
        return false;
      }
    },
    { message: "Must be a valid YouTube URL" },
  );

export function validateYoutubeUrl(url: string): { valid: boolean; error?: string } {
  const result = youtubeUrlSchema.safeParse(url);
  if (result.success) return { valid: true };
  return { valid: false, error: result.error.errors[0]?.message ?? "Invalid URL" };
}

export function extractYoutubeId(url: string): string | null {
  try {
    const u = new URL(url.trim());
    const host = u.hostname.replace(/^www\./, "");
    if (host === "youtu.be") {
      return u.pathname.slice(1).split("/")[0] || null;
    }
    if (host.includes("youtube.com")) {
      const v = u.searchParams.get("v");
      if (v) return v;
      const match = u.pathname.match(/\/(?:embed|shorts|live)\/([^/?]+)/);
      return match?.[1] ?? null;
    }
  } catch {
    return null;
  }
  return null;
}
