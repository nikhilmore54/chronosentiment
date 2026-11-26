/** Fail-closed freshness helpers — never invent healthy telemetry. */

export function isFiniteNumber(v: unknown): v is number {
  return typeof v === 'number' && Number.isFinite(v);
}

export function formatAgeSec(ageSec: number | null | undefined): string {
  if (!isFiniteNumber(ageSec)) return 'UNAVAILABLE';
  if (ageSec < 60) return `${Math.floor(ageSec)}s`;
  if (ageSec < 3600) return `${Math.floor(ageSec / 60)}m`;
  return `${(ageSec / 3600).toFixed(1)}h`;
}

export function ageFromMs(markMs: number | null | undefined, nowMs: number): number | null {
  if (!isFiniteNumber(markMs) || markMs <= 0) return null;
  return Math.max(0, (nowMs - markMs) / 1000);
}

export function staleLabel(
  ageSec: number | null | undefined,
  softSec = 30,
  hardSec = 300
): 'FRESH' | 'SOFT_STALE' | 'STALE' | 'UNAVAILABLE' {
  if (!isFiniteNumber(ageSec)) return 'UNAVAILABLE';
  if (ageSec <= softSec) return 'FRESH';
  if (ageSec <= hardSec) return 'SOFT_STALE';
  return 'STALE';
}

export function displayOrUnavailable(v: unknown, fallback = 'UNAVAILABLE'): string {
  if (v === null || v === undefined || v === '') return fallback;
  return String(v);
}

export function numOrNull(v: unknown): number | null {
  if (typeof v === 'number' && Number.isFinite(v)) return v;
  if (typeof v === 'string' && v.trim() !== '' && Number.isFinite(Number(v))) return Number(v);
  return null;
}
