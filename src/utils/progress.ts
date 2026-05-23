export function isCancelledError(e: unknown): boolean {
  const msg = e instanceof Error ? e.message : String(e);
  return /cancelled/i.test(msg);
}
