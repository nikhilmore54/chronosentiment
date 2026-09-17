/**
 * Live freshness for the Cockpit Now strip.
 *
 * Presentation only. Uses observation_feed status, last_ingest_unix, and
 * wall-clock time of the last ingest change seen by GET /deferred-live.
 * Does not decide, fill, or mark.
 */

import type { ObservationFeedSnapshot } from '../../types/decisionBrief';

/** Wall-clock bound: 1m bar + slack for an external live producer. */
export const EXTERNAL_LIVE_FRESHNESS_MS = 90_000;
/** Wall-clock bound while a controlled tape is PLAYING (poll interval is 4s). */
export const CONTROLLED_CLOCK_FRESHNESS_MS = 5_000;

export type LiveFreshnessKind = 'LIVE' | 'STALE' | 'TAPE EXHAUSTED' | 'WAITING FOR FEED';

export interface LiveFreshness {
  kind: LiveFreshnessKind;
  lastObsUnix: number | null;
  ageMs: number | null;
  boundMs: number;
  sourceLabel: string;
  speedLabel: string | null;
  clockSimulation: boolean;
}

function statusOf(feed: ObservationFeedSnapshot | null | undefined): string {
  return String(feed?.status ?? '').toUpperCase();
}

export function freshnessBoundMs(feed: ObservationFeedSnapshot | null | undefined): number {
  const src = String(feed?.observation_source ?? feed?.background ?? '').toUpperCase();
  if (feed?.clock_simulation || src === 'CACHED_1M' || src === 'YAHOO_1M') {
    return CONTROLLED_CLOCK_FRESHNESS_MS;
  }
  return EXTERNAL_LIVE_FRESHNESS_MS;
}

export function formatSpeedLabel(speed: number | null | undefined): string | null {
  if (speed == null || !Number.isFinite(speed)) return null;
  if (speed <= 0) return 'instant';
  const n = Number.isInteger(speed) ? String(speed) : speed.toFixed(1);
  return `${n}×`;
}

export function classifyLiveFreshness(
  feed: ObservationFeedSnapshot | null | undefined,
  lastIngestSeenAt: number | null,
  nowMs: number,
): LiveFreshness {
  const status = statusOf(feed);
  const lastObsUnix = feed?.last_ingest_unix ?? null;
  const boundMs = freshnessBoundMs(feed);
  const src = String(feed?.observation_source ?? feed?.background ?? 'NONE');
  const clockSimulation = feed?.clock_simulation === true
    || src === 'CACHED_1M'
    || src === 'YAHOO_1M';
  const speedLabel = formatSpeedLabel(feed?.speed);
  const ageMs = lastIngestSeenAt != null ? Math.max(0, nowMs - lastIngestSeenAt) : null;

  let kind: LiveFreshnessKind;
  if (status === 'WAITING_FOR_FEED' || status === 'WAITINGFORFEED') {
    kind = 'WAITING FOR FEED';
  } else if (status === 'TAPE_EXHAUSTED' || status === 'TAPEEXHAUSTED') {
    kind = 'TAPE EXHAUSTED';
  } else if (status === 'YAHOO_REFUSED' || status === 'YAHOOREFUSED') {
    kind = 'STALE';
  } else if (
    lastObsUnix != null
    && ageMs != null
    && ageMs <= boundMs
    && (status === 'PLAYING' || status === 'POLLING' || status === 'WAITING_FOR_INBOX' || status === 'WAITINGFORINBOX')
  ) {
    kind = 'LIVE';
  } else if (lastObsUnix != null && ageMs != null && ageMs <= boundMs && !clockSimulation) {
    kind = 'LIVE';
  } else if (lastObsUnix == null) {
    kind = 'WAITING FOR FEED';
  } else {
    kind = 'STALE';
  }

  return {
    kind,
    lastObsUnix,
    ageMs: kind === 'WAITING FOR FEED' ? null : ageMs,
    boundMs,
    sourceLabel: src,
    speedLabel,
    clockSimulation,
  };
}

export function formatObsAge(ageMs: number | null): string {
  if (ageMs == null) return '—';
  const sec = ageMs / 1000;
  if (sec < 10) return `${sec.toFixed(1)}s`;
  if (sec < 60) return `${Math.floor(sec)}s`;
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min}m`;
  const hr = Math.floor(min / 60);
  if (hr < 48) return `${hr}h`;
  return `${Math.floor(hr / 24)}d`;
}

export function freshnessColor(kind: LiveFreshnessKind): string {
  switch (kind) {
    case 'LIVE': return '#16a34a';
    case 'STALE': return '#d97706';
    case 'TAPE EXHAUSTED': return '#b45309';
    case 'WAITING FOR FEED': return '#6b7280';
  }
}
