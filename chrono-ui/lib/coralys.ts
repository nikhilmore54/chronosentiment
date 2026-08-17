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