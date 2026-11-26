/**
 * Read-only Pi client for Gate5 Operator.
 * Fetches `/pi-dashboard/api/state` via Vite proxy — no behavioural logic.
 */

import { mapOperatorState } from './mapState';
import type { OperatorState } from './types';

const STATE_URL = '/pi-dashboard/api/state';

export async function fetchOperatorState(signal?: AbortSignal): Promise<OperatorState> {
  const fetched_at_ms = Date.now();
  try {
    const res = await fetch(STATE_URL, {
      method: 'GET',
      headers: { Accept: 'application/json' },
      signal,
      cache: 'no-store',
    });
    if (!res.ok) {
      return mapOperatorState(null, {
        fetch_status: 'error',
        fetch_error: `HTTP ${res.status}`,
        fetched_at_ms,
      });
    }
    const json = (await res.json()) as Record<string, unknown>;
    if (!json || typeof json !== 'object') {
      return mapOperatorState(null, {
        fetch_status: 'unavailable',
        fetch_error: 'Empty or invalid JSON from Pi /api/state',
        fetched_at_ms,
      });
    }
    return mapOperatorState(json, {
      fetch_status: 'ok',
      fetch_error: null,
      fetched_at_ms,
    });
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    const aborted = e instanceof Error && e.name === 'AbortError';
    return mapOperatorState(null, {
      fetch_status: aborted ? 'unavailable' : 'error',
      fetch_error: aborted ? 'Request aborted' : msg,
      fetched_at_ms,
    });
  }
}
