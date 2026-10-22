import type { DateRange } from "./DateRangePicker";

export const OPEN_ANALYST_DAY_KEY = "desk:analyst:open-day";
export const OPEN_DAY_MAX_AGE_MS = 2 * 60_000;

export interface OpenDayRequest {
  day: string;
  requestedAtMs: number;
  requestId: string;
}

function isCalendarDay(value: unknown): value is string {
  if (typeof value !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const [year, month, day] = value.split("-").map(Number);
  const date = new Date(Date.UTC(year, month - 1, day));
  return date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day;
}

export function localCalendarDay(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function shiftCalendarDay(day: string, amount: number): string {
  if (!isCalendarDay(day) || !Number.isInteger(amount)) throw new RangeError("Invalid calendar day shift");
  const [year, month, date] = day.split("-").map(Number);
  const shifted = new Date(Date.UTC(year, month - 1, date + amount));
  return `${shifted.getUTCFullYear()}-${String(shifted.getUTCMonth() + 1).padStart(2, "0")}-${String(shifted.getUTCDate()).padStart(2, "0")}`;
}

export function createOpenDayRequest(day: string, requestedAtMs: number, requestId: string): OpenDayRequest {
  if (!isCalendarDay(day) || !Number.isFinite(requestedAtMs) || !requestId) {
    throw new RangeError("Invalid selected-day request");
  }
  return { day, requestedAtMs, requestId };
}

export function parseOpenDayRequest(
  raw: string | null,
  nowMs = Date.now(),
): OpenDayRequest | null {
  if (!raw) return null;
  try {
    const value: unknown = JSON.parse(raw);
    if (!value || typeof value !== "object") return null;
    const request = value as Partial<OpenDayRequest>;
    if (!isCalendarDay(request.day) || !Number.isFinite(request.requestedAtMs) ||
        typeof request.requestId !== "string" || !request.requestId) return null;
    const age = nowMs - request.requestedAtMs!;
    if (age < -30_000 || age > OPEN_DAY_MAX_AGE_MS) return null;
    return request as OpenDayRequest;
  } catch {
    return null;
  }
}

function calendarDayIndex(day: string): number | null {
  if (!isCalendarDay(day)) return null;
  const [year, month, date] = day.split("-").map(Number);
  return Date.UTC(year, month - 1, date) / 86_400_000;
}

export function rangeForOpenDay(range: DateRange, day: string): DateRange {
  const target = calendarDayIndex(day);
  const from = calendarDayIndex(range.from);
  const to = calendarDayIndex(range.to);
  if (target === null) return range;
  if (from !== null && to !== null && from <= target && target <= to) return range;
  const span = from !== null && to !== null && to >= from ? to - from : 0;
  return { from: shiftCalendarDay(day, -span), to: day };
}
