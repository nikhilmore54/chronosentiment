/**
 * Decision API client — IC v1 Intraday Decision API.
 *
 * Wraps the Rust chronosentiment_server endpoints.
 * The UI never reads raw datasets or runs intelligence logic.
 *
 * Base URL is configurable via VITE_API_BASE env var (default: http://localhost:8080).
 */

import type { DecisionBrief, DecisionListResponse, PortfolioContextResponse, RecommendationFeedResponse, LifecycleFeedResponse, PaperBlotter, DeferredLiveResponse } from '../types/decisionBrief';

const API_BASE = (import.meta.env.VITE_API_BASE as string | undefined) ?? 'http://localhost:8080';

export interface SearchParams {
  ticker?: string;
  direction?: string;
  entry_state?: string;
  h120_state?: string;
  action?: string;
  date?: string;
}

/**
 * GET /api/v1/intraday/decisions
 * Returns all enriched decisions from the Rust IC v1 engine.
 */
export async function fetchAllDecisions(): Promise<DecisionListResponse> {
  const res = await fetch(`${API_BASE}/api/v1/intraday/decisions`);
  if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
  return res.json() as Promise<DecisionListResponse>;
}

/**
 * GET /api/v1/intraday/decisions/search?ticker=...
 * Returns filtered decisions. All params optional.
 */
export async function searchDecisions(params: SearchParams): Promise<DecisionListResponse> {
  const qs = new URLSearchParams();
  if (params.ticker)      qs.set('ticker',      params.ticker);
  if (params.direction)   qs.set('direction',   params.direction);
  if (params.entry_state) qs.set('entry_state', params.entry_state);
  if (params.h120_state)  qs.set('h120_state',  params.h120_state);
  if (params.action)      qs.set('action',      params.action);
  if (params.date)        qs.set('date',        params.date);

  const url = `${API_BASE}/api/v1/intraday/decisions/search${qs.toString() ? '?' + qs.toString() : ''}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
  return res.json() as Promise<DecisionListResponse>;
}

/**
 * GET /api/v1/intraday/decisions/:id
 * Returns a single decision by decision_id.
 */
export async function fetchDecisionById(id: string): Promise<DecisionBrief> {
  const res = await fetch(`${API_BASE}/api/v1/intraday/decisions/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
  return res.json() as Promise<DecisionBrief>;
}

/**
 * GET /api/v1/intraday/portfolio-context
 * Returns all IC v1 ACT decisions with portfolio context:
 * OQS rank within the day, selected (top-5 by OQS), and position size.
 * Mirrors the Backtest v2 OQS-ranked selection logic.
 */
export async function fetchPortfolioContext(): Promise<PortfolioContextResponse> {
  const res = await fetch(`${API_BASE}/api/v1/intraday/portfolio-context`);
  if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
  return res.json() as Promise<PortfolioContextResponse>;
}

/**
 * GET /api/v1/intraday/recommendations
 * Returns the top-5 selected ACT decisions per date as recommendation cards.
 * Primary data source for the Decision Feed in the Cockpit UI.
 */
export async function fetchRecommendations(): Promise<RecommendationFeedResponse> {
  const res = await fetch(`${API_BASE}/api/v1/intraday/recommendations`);
  if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
  return res.json() as Promise<RecommendationFeedResponse>;
}

/**
 * GET /api/v1/intraday/position-lifecycle
 * Returns top-5 ACT decisions per date enriched with lifecycle status
 * from the reassessment v0.2 replay (NEW | REASSESS_EXIT | CLOSED).
 */
export async function fetchPositionLifecycle(): Promise<LifecycleFeedResponse> {
  const res = await fetch(`${API_BASE}/api/v1/intraday/position-lifecycle`);
  if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
  return res.json() as Promise<LifecycleFeedResponse>;
}

/**
 * GET /api/v1/intraday/paper-trades
 * Frozen Paper Trader v0.2 replay blotter.
 */
export async function fetchPaperTrades(): Promise<PaperBlotter> {
  const res = await fetch(`${API_BASE}/api/v1/intraday/paper-trades`);
  if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
  return res.json() as Promise<PaperBlotter>;
}

/**
 * GET /api/v1/intraday/deferred-live
 * Live paper ledger. Distinct from frozen /paper-trades replay.
 */
export async function fetchDeferredLive(): Promise<DeferredLiveResponse> {
  const res = await fetch(`${API_BASE}/api/v1/intraday/deferred-live`);
  if (!res.ok) throw new Error(`API error ${res.status}: ${res.statusText}`);
  return res.json() as Promise<DeferredLiveResponse>;
}