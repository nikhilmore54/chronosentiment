/**
 * Coralys Decision Intelligence — API types and client.
 *
 * These types mirror the `coralys_decision_server` response shapes exactly.
 * No confidence, probability, ranking, or allocation fields are present.
 *
 * API base: CORALYS_API_URL env var (default: http://localhost:3001)
 */

// ─── Enums ────────────────────────────────────────────────────────────────────

export type Direction = "LONG" | "SHORT" | "NO_TRADE";
export type CertificationStatus = "CERTIFIED" | "PENDING" | "FAILED";
export type ExecutionStatus =
  | "NOT_RECORDED"
  | "USER_IGNORED"
  | "USER_EXECUTED"
  | "USER_CANCELLED";
export type OutcomeStatus =
  | "OPEN"
  | "TARGET"
  | "REFERENCE_RISK"
  | "HORIZON"
  | "USER_CLOSED";

// ─── Full DecisionRecord response ─────────────────────────────────────────────

export interface CoralysIdentity {
  decision_id: string;
  instrument: string;
  decision_timestamp: string; // ISO 8601
}

export interface CoralysCertification {
  status: CertificationStatus;
  policy_artifact_hash: string;
  execution_artifact_hash: string | null;
  decision_pipeline: string;
  certified_timestamp: string;
  data_snapshot_id: string;
}

export interface CoralysDecisionCore {
  direction: Direction;
  trend: string;
  momentum: string;
  volatility: string;
  target_price: number | null;
  /** ATR-14 in price units at decision time T. Null when unavailable. */
  atr_14: number | null;
  /** Last traded price / previous close at decision time T. Label as "LTP / Reference" until execution. */
  reference_price: number | null;
  /** Next NSE trading session date (YYYY-MM-DD) this decision applies to. */
  effective_session: string | null;
}

export interface CoralysReferenceRisk {
  boundary_price: number | null;
  boundary_type: string;
  status: "REFERENCE";
}

export interface CoralysExecution {
  status: ExecutionStatus;
  execution_timestamp: string | null;
  quantity: number | null;
  execution_price: number | null;
  execution_source: string | null;
}

export interface CoralysOutcome {
  status: OutcomeStatus;
  exit_reason: string | null;
  exit_timestamp: string | null;
  exit_price: number | null;
  realized_pnl: number | null;
}

export interface CoralysEvidence {
  similar_decisions_count: number | null;
  historical_target_rate: number | null;
  median_mae_pct: number | null;
  p90_mae_pct: number | null;
  median_mfe_pct: number | null;
  median_time_to_target_sessions: number | null;
}

export interface CoralysDecision {
  identity: CoralysIdentity;
  certification: CoralysCertification;
  decision: CoralysDecisionCore;
  reference_risk: CoralysReferenceRisk;
  execution: CoralysExecution;
  outcome: CoralysOutcome;
  evidence: CoralysEvidence;
}

// ─── Feed response ────────────────────────────────────────────────────────────

export interface FeedEntry {
  decision_id: string;
  instrument: string;
  decision_timestamp: string;
  direction: Direction;
  certification_status: CertificationStatus;
  target_price: number | null;
  reference_risk_boundary_price: number | null;
  reference_risk_boundary_type: string;
  outcome_status: string;
  execution_status: string;
  /** ATR-14 in price units at decision time T. Null when unavailable. */
  atr_14: number | null;
  /** Last traded price / previous close at decision time T. */
  reference_price: number | null;
  /** Next NSE trading session date (YYYY-MM-DD) this decision applies to. */
  effective_session: string | null;
  /** Trend label from certified TMV state. */
  trend: string;
  /** Momentum label from certified TMV state. */
  momentum: string;
}

export interface FeedResponse {
  decisions: FeedEntry[];
  total: number;
}

// ─── Detail response ──────────────────────────────────────────────────────────

export interface DetailResponse {
  decision: CoralysDecision;
}

// ─── API client ───────────────────────────────────────────────────────────────

function apiBase(): string {
  return process.env.CORALYS_API_URL ?? "http://localhost:3001";
}

/**
 * Fetch the Decision Feed from the Coralys Decision Intelligence API.
 * Returns an empty feed on error (server may not be running in dev).
 */
export async function fetchDecisionFeed(): Promise<FeedResponse> {
  try {
    const res = await fetch(`${apiBase()}/decisions`, {
      cache: "no-store",
    });
    if (!res.ok) return { decisions: [], total: 0 };
    return (await res.json()) as FeedResponse;
  } catch {
    return { decisions: [], total: 0 };
  }
}

/**
 * Fetch a single certified decision by ID.
 * Returns null if not found or on error.
 */
export async function fetchDecision(id: string): Promise<CoralysDecision | null> {
  try {
    const res = await fetch(`${apiBase()}/decisions/${encodeURIComponent(id)}`, {
      cache: "no-store",
    });
    if (!res.ok) return null;
    const body = (await res.json()) as DetailResponse;
    return body.decision;
  } catch {
    return null;
  }
}

// ─── Formatting helpers ───────────────────────────────────────────────────────

export function shortHash(s: string): string {
  // Take the last segment after the last '-' if it looks like a hash, else first 8 chars.
  const parts = s.split("-");
  const last = parts[parts.length - 1];
  if (last.length >= 8 && /^[0-9a-f]+$/i.test(last)) return last.slice(0, 8);
  return s.slice(0, 8);
}

export function formatDecisionTime(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleTimeString("en-IN", {
      hour: "2-digit",
      minute: "2-digit",
      timeZone: "Asia/Kolkata",
    });
  } catch {
    return iso;
  }
}

export function formatDecisionDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString("en-IN", {
      day: "numeric",
      month: "short",
      year: "numeric",
      timeZone: "Asia/Kolkata",
    });
  } catch {
    return iso;
  }
}

export function formatPrice(n: number | null): string {
  if (n === null) return "—";
  return `₹${n.toLocaleString("en-IN", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function directionLabel(d: Direction): string {
  return d === "NO_TRADE" ? "NO TRADE" : d;
}

// ─── Indicative price computation ────────────────────────────────────────────
//
// Mirrors `compute_execution_params` in `coralys_execution_model.rs`.
// FROZEN DESIGN PARAMETERS — must not be changed without updating the Rust model.
// These are used ONLY for pre-execution "Indicative" display.
// After actual execution, target/risk are computed from the actual fill price.

const TARGET_PCT_MIN = 0.02;
const TARGET_PCT_MAX = 0.15;
const RISK_PCT_MIN = 0.01;
const RISK_PCT_MAX = 0.08;

function tmvMultipliers(trend: string, momentum: string): { target: number; risk: number } {
  const state = `${trend}_${momentum}`;
  switch (state) {
    case "Bullish_Positive": return { target: 2.0, risk: 1.0 };
    case "Bullish_Negative": return { target: 1.5, risk: 0.75 };
    case "Bearish_Positive": return { target: 1.5, risk: 0.75 };
    case "Bearish_Negative": return { target: 1.0, risk: 0.5 };
    default:                 return { target: 1.0, risk: 0.5 };
  }
}

export interface IndicativePrices {
  indicative_target: number;
  indicative_risk: number;
  upside_pct: number;
  downside_pct: number;
}

/**
 * Compute indicative target and reference-risk prices from ATR-14 and TMV state.
 *
 * Uses the reference_price (LTP at decision time) as the indicative entry.
 * Returns null if ATR-14 or reference_price is unavailable.
 *
 * IMPORTANT: These are INDICATIVE only. After actual execution, target/risk
 * must be recomputed from the actual fill price using the canonical Rust model.
 */
export function computeIndicativePrices(
  entry: number | null,
  atr_14: number | null,
  trend: string,
  momentum: string,
  direction: Direction,
): IndicativePrices | null {
  if (!entry || !atr_14 || entry <= 0 || atr_14 <= 0) return null;
  if (direction === "NO_TRADE") return null;

  const { target: tMul, risk: rMul } = tmvMultipliers(trend, momentum);
  const base = atr_14 / entry;
  const target_pct = Math.min(Math.max(base * tMul, TARGET_PCT_MIN), TARGET_PCT_MAX);
  const risk_pct   = Math.min(Math.max(base * rMul, RISK_PCT_MIN),   RISK_PCT_MAX);

  const indicative_target = direction === "LONG"
    ? entry * (1 + target_pct)
    : entry * (1 - target_pct);
  const indicative_risk = direction === "LONG"
    ? entry * (1 - risk_pct)
    : entry * (1 + risk_pct);

  const upside_pct   = direction === "LONG" ? target_pct : risk_pct;
  const downside_pct = direction === "LONG" ? risk_pct   : target_pct;

  return { indicative_target, indicative_risk, upside_pct, downside_pct };
}

export function formatPct(n: number): string {
  return `${n >= 0 ? "+" : ""}${(n * 100).toFixed(2)}%`;
}