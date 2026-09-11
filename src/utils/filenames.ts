const FILENAME_MAX_LENGTH = 80;
const RESERVED_CHARS = /[<>:"/\\|?*\u0000-\u001f]/g;

export function sanitizeFilename(title: string, fallback: string): string {
  const cleaned = title
    .replace(RESERVED_CHARS, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/[. ]+$/g, "")
    .slice(0, FILENAME_MAX_LENGTH)
    .replace(/[. ]+$/g, "");

  return cleaned || fallback;
}

export function getDirectory(path: string): string | null {
  const sepIndex = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  if (sepIndex <= 0) return null;
  const parent = path.slice(0, sepIndex);
  return parent || null;
}
