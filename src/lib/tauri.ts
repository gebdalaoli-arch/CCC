declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export async function invokeOrFallback<T>(
  cmd: string,
  args: Record<string, unknown> | undefined,
  fallback: T
): Promise<T> {
  const canInvoke =
    typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);

  if (!canInvoke) {
    return fallback;
  }

  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}
