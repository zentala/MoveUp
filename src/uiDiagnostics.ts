import { error as logError } from "@tauri-apps/plugin-log";

const SAFE_ERROR_KINDS = new Set([
  "Error",
  "TypeError",
  "RangeError",
  "ReferenceError",
  "SyntaxError",
  "URIError",
  "EvalError",
  "AggregateError",
  "DOMException",
  "NonError",
  "UnknownError",
]);

type FailureSource = "render" | "window" | "rejection";

type SafeLocation = {
  module: string;
  line?: number;
  column?: number;
};

function errorKind(value: unknown): string {
  try {
    if (value instanceof DOMException) return "DOMException";
    if (!(value instanceof Error)) return "NonError";
    return SAFE_ERROR_KINDS.has(value.name) ? value.name : "Error";
  } catch {
    return "UnknownError";
  }
}

function safeCoordinate(value: number): number | undefined {
  return Number.isSafeInteger(value) && value > 0 && value <= 1_000_000_000
    ? value
    : undefined;
}

function safeModuleBasename(value: string): string | undefined {
  try {
    const current = new URL(window.location.href);
    const candidate = new URL(value, current);
    if (
      current.origin === "null" ||
      candidate.origin !== current.origin ||
      candidate.protocol !== current.protocol ||
      candidate.host !== current.host
    ) {
      return undefined;
    }

    const basename = candidate.pathname.split("/").pop() ?? "";
    if (
      !/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}\.(?:c|m)?(?:js|jsx|ts|tsx)$/i.test(
        basename,
      )
    ) {
      return undefined;
    }
    if (!/^\/(?:src|assets)\/.+/i.test(candidate.pathname)) {
      return undefined;
    }
    return basename;
  } catch {
    return undefined;
  }
}

function locationFromUrl(
  file: string,
  line: number,
  column: number,
): SafeLocation | undefined {
  const module = safeModuleBasename(file);
  if (!module) return undefined;

  const safeLine = safeCoordinate(line);
  const safeColumn = safeCoordinate(column);
  return safeLine && safeColumn
    ? { module, line: safeLine, column: safeColumn }
    : { module };
}

function locationFromStack(value: unknown): SafeLocation | undefined {
  try {
    if (!(value instanceof Error) || typeof value.stack !== "string") {
      return undefined;
    }

    const stackLines = value.stack.split(/\r?\n/);
    const message = typeof value.message === "string" ? value.message : "";
    const messageLines = message.split(/\r?\n/);
    const firstMessageLine = messageLines[0] ?? "";
    const header = stackLines[0] ?? "";
    const hasV8Header =
      header === `${value.name}: ${firstMessageLine}` ||
      (message.length === 0 && header === value.name);
    const hasBareHeader = header === firstMessageLine;
    const frames =
      hasV8Header || hasBareHeader
        ? stackLines.slice(messageLines.length)
        : stackLines;

    for (const frame of frames) {
      const match =
        frame.match(
          /^\s*at\s+(?:.+?\s+\()?((?:https?|tauri|file):\/\/[^\s)]+):(\d+):(\d+)\)?\s*$/,
        ) ??
        frame.match(
          /^\s*[^@\s]+@((?:https?|tauri|file):\/\/\S+):(\d+):(\d+)\s*$/,
        );
      if (!match) continue;
      const location = locationFromUrl(
        match[1],
        Number(match[2]),
        Number(match[3]),
      );
      if (location?.line && location.column) return location;
    }
  } catch {
    return undefined;
  }
  return undefined;
}

function writeDiagnostic(
  source: FailureSource,
  value: unknown,
  location?: SafeLocation,
): void {
  const fields = ["UI_FAILURE", `source=${source}`, `kind=${errorKind(value)}`];
  if (location) fields.push(`module=${location.module}`);
  if (location?.line && location.column) {
    fields.push(`line=${location.line}`, `column=${location.column}`);
  }

  try {
    void logError(fields.join(" ")).catch(() => undefined);
  } catch {
    // Diagnostics are best-effort; logging failures must not escape into the UI.
  }
}

function locationFromErrorEvent(event: ErrorEvent): SafeLocation | undefined {
  return locationFromStack(event.error);
}

export function reportRenderFailure(error: unknown): void {
  writeDiagnostic("render", error, locationFromStack(error));
}

export function startUiDiagnostics(): () => void {
  const onError = (event: ErrorEvent): void => {
    writeDiagnostic("window", event.error, locationFromErrorEvent(event));
  };

  const onUnhandledRejection = (event: PromiseRejectionEvent): void => {
    writeDiagnostic(
      "rejection",
      event.reason,
      locationFromStack(event.reason),
    );
  };

  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onUnhandledRejection);

  return () => {
    window.removeEventListener("error", onError);
    window.removeEventListener("unhandledrejection", onUnhandledRejection);
  };
}
