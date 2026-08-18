"use client";

/**
 * /live — Trading Home Screen
 *
 * Fetches the ranked RecommendationSnapshot from GET /recommendations/latest
 * (coralys_decision_server :3001) and displays it as the primary trading
 * home screen.
 *
 * Architecture contract:
 * - ALL recommendation logic lives in the Rust engine (coralys-decision).
 * - This page displays RecommendationRecord fields verbatim — no re-ranking,
 *   no re-scoring, no probability claims.
 * - Historical target-before-risk rates are NOT presented as forward
 *   probabilities of success.
 */

import { useEffect, useState } from "react";
import Link from "next/link";

// ─── Types (mirror RecommendationRecord from the Rust engine) ─────────────────

interface AnalogueKey {
  direction: string;
  coralys_state: string;
  narrow_match: boolean;
  sample_size: number;
}

interface HistoricalEvidence {
  analogue_key: AnalogueKey;
  sample_size: number;
  target_before_risk_rate: number;
  risk_before_target_rate: number;
  horizon_rate: number;
  median_mfe: number;
  median_mae: number;
  median_sessions_to_target: number | null;
  evidence_class: "Favourable" | "Mixed" | "Unfavourable" | "Insufficient";
}

interface ScoreComponents {
  evidence_weight: number;
  rr_weight: number;
  freshness_weight: number;
  evidence_contribution: number;
  rr_contribution: number;
  freshness_contribution: number;
}

interface RecommendationRecord {
  decision_id: string;
  instrument: string;
  direction: string;
  trend: string;
  momentum: string;
  reference_price: number | null;
  atr_14: number | null;
  indicative_target: number | null;
  indicative_risk: number | null;
  upside_pct: number | null;
  downside_pct: number | null;
  rr: number | null;
  horizon_min_sessions: number;
  horizon_max_sessions: number;
  effective_session: string | null;
  evidence: HistoricalEvidence;
  action: "Buy" | "Watch" | "NoTrade";
  rank_score: number;
  recommendation_policy_version: string;
  score_components: ScoreComponents;
}

interface RecommendationSnapshot {
  evaluated: number;
  actionable: number;
  recommendations: RecommendationRecord[];
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function fmt(n: number | null, decimals = 2): string {
  if (n === null || n === undefined) return "—";
  return n.toLocaleString("en-IN", { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
}

function fmtPct(n: number | null): string {
  if (n === null || n === undefined) return "—";
  return (n * 100).toFixed(2) + "%";
}

function actionColor(action: RecommendationRecord["action"]): string {
  switch (action) {
    case "Buy": return "#10b981";
    case "Watch": return "#f59e0b";
    case "NoTrade": return "#6b7280";
  }
}

function actionLabel(action: RecommendationRecord["action"]): string {
  switch (action) {
    case "Buy": return "BUY";
    case "Watch": return "WATCH";
    case "NoTrade": return "NO TRADE";
  }
}

function evidenceColor(cls: HistoricalEvidence["evidence_class"]): string {
  switch (cls) {
    case "Favourable": return "#10b981";
    case "Mixed": return "#f59e0b";
    case "Unfavourable": return "#ef4444";
    case "Insufficient": return "#6b7280";
  }
}

function directionClass(dir: string): string {
  if (dir === "LONG") return "badge badge-long";
  if (dir === "SHORT") return "badge badge-short";
  return "badge badge-no-trade";
}

// ─── Recommendation Card ──────────────────────────────────────────────────────

function RecommendationCard({ rec }: { rec: RecommendationRecord }) {
  const ac = actionColor(rec.action);
  const ec = evidenceColor(rec.evidence.evidence_class);
  const isBuy = rec.action === "Buy";
  const isWatch = rec.action === "Watch";

  return (
    <div style={{
      background: "var(--bg-card)",
      border: `1px solid ${isBuy ? "rgba(16,185,129,0.25)" : isWatch ? "rgba(245,158,11,0.2)" : "var(--border)"}`,
      borderRadius: "12px",
      overflow: "hidden",
    }}>
      {/* Top accent bar — action colour */}
      <div style={{ height: "3px", background: ac }} />

      <div style={{ padding: "16px 20px" }}>
        {/* Header row */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "14px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <span style={{ fontSize: "15px", fontWeight: "700", color: "var(--text-primary)" }}>{rec.instrument}</span>
            <span className={directionClass(rec.direction)}>{rec.direction}</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            {isBuy && <div style={{ width: "6px", height: "6px", borderRadius: "50%", background: "#10b981" }} className="animate-pulse-slow" />}
            <span style={{
              fontSize: "11px", fontWeight: "700", color: ac,
              background: `${ac}18`, padding: "2px 8px", borderRadius: "4px", letterSpacing: "0.05em",
            }}>
              {actionLabel(rec.action)}
            </span>
          </div>
        </div>

        {/* State + session */}
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px", marginBottom: "12px" }}>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Coralys State</div>
            <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
              {rec.trend} / {rec.momentum}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Session</div>
            <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
              {rec.effective_session ?? "—"}
            </div>
          </div>
        </div>

        {/* Geometry row */}
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: "10px", marginBottom: "12px" }}>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Reference</div>
            <div style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-primary)" }}>
              {fmt(rec.reference_price)}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Ind. Target</div>
            <div style={{ fontSize: "13px", fontWeight: "700", color: "#10b981" }}>
              {fmt(rec.indicative_target)}
              {rec.upside_pct !== null && (
                <span style={{ fontSize: "10px", color: "#10b981", marginLeft: "4px" }}>
                  {rec.direction === "SHORT" ? "-" : "+"}{fmtPct(rec.upside_pct)}
                </span>
              )}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Ind. Risk</div>
            <div style={{ fontSize: "13px", fontWeight: "700", color: "#ef4444" }}>
              {fmt(rec.indicative_risk)}
              {rec.downside_pct !== null && (
                <span style={{ fontSize: "10px", color: "#ef4444", marginLeft: "4px" }}>
                  {rec.direction === "SHORT" ? "+" : "-"}{fmtPct(rec.downside_pct)}
                </span>
              )}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>R:R</div>
            <div style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-primary)" }}>
              {rec.rr !== null ? rec.rr.toFixed(2) : "—"}
            </div>
          </div>
        </div>

        {/* Evidence row */}
        <div style={{
          padding: "10px 12px",
          background: `${ec}0d`,
          border: `1px solid ${ec}25`,
          borderRadius: "8px",
          marginBottom: "10px",
        }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "6px" }}>
            <span style={{ fontSize: "10px", fontWeight: "700", color: ec, letterSpacing: "0.05em" }}>
              {rec.evidence.evidence_class.toUpperCase()} EVIDENCE
            </span>
            <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>
              n={rec.evidence.sample_size}
              {!rec.evidence.analogue_key.narrow_match && (
                <span style={{ marginLeft: "4px", color: "#f59e0b" }}>(broad)</span>
              )}
            </span>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "8px" }}>
            <div>
              <div style={{ fontSize: "9px", color: "var(--text-muted)", marginBottom: "1px" }}>Target rate</div>
              <div style={{ fontSize: "12px", fontWeight: "700", color: ec }}>
                {fmtPct(rec.evidence.target_before_risk_rate)}
              </div>
            </div>
            <div>
              <div style={{ fontSize: "9px", color: "var(--text-muted)", marginBottom: "1px" }}>Median MFE</div>
              <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
                {fmtPct(rec.evidence.median_mfe)}
              </div>
            </div>
            <div>
              <div style={{ fontSize: "9px", color: "var(--text-muted)", marginBottom: "1px" }}>Median MAE</div>
              <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
                {fmtPct(rec.evidence.median_mae)}
              </div>
            </div>
          </div>
          <div style={{ marginTop: "6px", fontSize: "9px", color: "var(--text-muted)", fontStyle: "italic" }}>
            Historical rates describe past analogues — not forward probabilities of success.
          </div>
        </div>

        {/* Footer: rank score + link */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>
            Rank score: <span style={{ color: "var(--text-secondary)", fontWeight: "600" }}>{rec.rank_score.toFixed(4)}</span>
            <span style={{ marginLeft: "6px" }}>· policy {rec.recommendation_policy_version}</span>
          </div>
          <Link
            href={`/decisions/${rec.decision_id}`}
            style={{ fontSize: "11px", color: "var(--text-muted)", textDecoration: "none" }}
          >
            Decision →
          </Link>
        </div>
      </div>
    </div>
  );
}

// ─── Market status (IST) ──────────────────────────────────────────────────────

function getMarketStatus() {
  const now = new Date();
  const istOffset = 5.5 * 60 * 60 * 1000;
  const ist = new Date(now.getTime() + istOffset);
  const day = ist.getUTCDay();
  const h = ist.getUTCHours();
  const m = ist.getUTCMinutes();
  const mins = h * 60 + m;
  const open = 9 * 60 + 15;
  const close = 15 * 60 + 30;
  if (day === 0 || day === 6) return { status: "CLOSED", label: "Weekend", color: "#6b7280", next: "Monday 09:15 IST" };
  if (mins < open) return { status: "PRE-MARKET", label: "Pre-market", color: "#f59e0b", next: "09:15 IST today" };
  if (mins >= open && mins < close) return { status: "OPEN", label: "Market open", color: "#10b981", next: "Closes 15:30 IST" };
  return { status: "CLOSED", label: "Market closed", color: "#6b7280", next: "Tomorrow 09:15 IST" };
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function LivePage() {
  const [snapshot, setSnapshot] = useState<RecommendationSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const market = getMarketStatus();

  useEffect(() => {
    fetch("/api/recommendations/latest")
      .then((r) => {
        if (!r.ok) throw new Error(`Server returned ${r.status}`);
        return r.json() as Promise<RecommendationSnapshot>;
      })
      .then((data) => {
        setSnapshot(data);
        setLoading(false);
      })
      .catch((e: Error) => {
        setError(e.message);
        setLoading(false);
      });
  }, []);

  const buyRecs = snapshot?.recommendations.filter((r) => r.action === "Buy") ?? [];
  const watchRecs = snapshot?.recommendations.filter((r) => r.action === "Watch") ?? [];
  const noTradeRecs = snapshot?.recommendations.filter((r) => r.action === "NoTrade") ?? [];

  return (
    <div style={{ maxWidth: "1200px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", flexWrap: "wrap", gap: "16px", marginBottom: "28px" }}>
        <div>
          <p style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>ChronoSentiment</p>
          <h1 style={{ fontSize: "24px", fontWeight: "700", color: "var(--text-primary)", letterSpacing: "-0.02em", margin: "0 0 6px 0" }}>Recommendations</h1>
          <p style={{ fontSize: "13px", color: "var(--text-secondary)", margin: 0 }}>
            NSE · Ranked by evidence + geometry · Policy {snapshot?.recommendations[0]?.recommendation_policy_version ?? "v0"}
          </p>
        </div>
        <div style={{ display: "flex", gap: "10px", alignItems: "center", flexWrap: "wrap" }}>
          <div style={{ padding: "10px 16px", background: "var(--bg-card)", border: `1px solid ${market.color}30`, borderRadius: "8px", display: "flex", alignItems: "center", gap: "8px" }}>
            <div style={{ width: "8px", height: "8px", borderRadius: "50%", background: market.color }} />
            <div>
              <div style={{ fontSize: "12px", fontWeight: "700", color: market.color }}>{market.status}</div>
              <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>{market.next}</div>
            </div>
          </div>
          <div style={{ padding: "4px 10px", background: "rgba(245,158,11,0.1)", border: "1px solid rgba(245,158,11,0.2)", borderRadius: "4px", fontSize: "11px", fontWeight: "700", color: "#f59e0b", letterSpacing: "0.05em" }}>
            PAPER ONLY
          </div>
        </div>
      </div>

      {/* Summary stats */}
      {snapshot && (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "12px", marginBottom: "28px" }}>
          {[
            { label: "Evaluated", value: snapshot.evaluated, color: "var(--text-secondary)" },
            { label: "Actionable", value: snapshot.actionable, color: "#10b981" },
            { label: "BUY", value: buyRecs.length, color: "#10b981" },
            { label: "WATCH", value: watchRecs.length, color: "#f59e0b" },
          ].map((s) => (
            <div key={s.label} style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "10px", padding: "14px 16px" }}>
              <div style={{ fontSize: "10px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "6px" }}>{s.label}</div>
              <div style={{ fontSize: "24px", fontWeight: "700", color: s.color }}>{s.value}</div>
            </div>
          ))}
        </div>
      )}

      {/* Loading */}
      {loading && (
        <div style={{ padding: "48px", textAlign: "center", color: "var(--text-muted)", fontSize: "13px" }}>
          Loading recommendations from Decision Server…
        </div>
      )}

      {/* Error */}
      {error && (
        <div style={{ padding: "20px", background: "rgba(239,68,68,0.06)", border: "1px solid rgba(239,68,68,0.2)", borderRadius: "10px", marginBottom: "24px" }}>
          <div style={{ fontSize: "13px", fontWeight: "600", color: "#ef4444", marginBottom: "4px" }}>Decision Server unavailable</div>
          <div style={{ fontSize: "12px", color: "var(--text-muted)" }}>{error}</div>
          <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "8px" }}>
            Start the server: <code style={{ background: "var(--bg-secondary)", padding: "1px 6px", borderRadius: "3px" }}>cargo run -p coralys_decision_server</code>
          </div>
        </div>
      )}

      {/* BUY recommendations */}
      {buyRecs.length > 0 && (
        <div style={{ marginBottom: "28px" }}>
          <h2 style={{ fontSize: "13px", fontWeight: "600", color: "#10b981", letterSpacing: "0.06em", textTransform: "uppercase", margin: "0 0 12px 0" }}>
            BUY — {buyRecs.length} decision{buyRecs.length !== 1 ? "s" : ""}
          </h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(360px, 1fr))", gap: "12px" }}>
            {buyRecs.map((r) => <RecommendationCard key={r.decision_id} rec={r} />)}
          </div>
        </div>
      )}

      {/* WATCH recommendations */}
      {watchRecs.length > 0 && (
        <div style={{ marginBottom: "28px" }}>
          <h2 style={{ fontSize: "13px", fontWeight: "600", color: "#f59e0b", letterSpacing: "0.06em", textTransform: "uppercase", margin: "0 0 12px 0" }}>
            WATCH — {watchRecs.length} decision{watchRecs.length !== 1 ? "s" : ""}
          </h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(360px, 1fr))", gap: "12px" }}>
            {watchRecs.map((r) => <RecommendationCard key={r.decision_id} rec={r} />)}
          </div>
        </div>
      )}

      {/* NO TRADE — collapsed by default, shown as count only */}
      {noTradeRecs.length > 0 && (
        <div style={{ marginBottom: "24px" }}>
          <div style={{ padding: "12px 16px", background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "8px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <span style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.06em", textTransform: "uppercase" }}>
              NO TRADE — {noTradeRecs.length} decision{noTradeRecs.length !== 1 ? "s" : ""}
            </span>
            <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
              Unfavourable, Insufficient, or NO_TRADE direction
            </span>
          </div>
        </div>
      )}

      {/* Empty state */}
      {!loading && !error && snapshot && snapshot.evaluated === 0 && (
        <div style={{ padding: "48px", textAlign: "center", color: "var(--text-muted)", fontSize: "13px" }}>
          No certified decisions in the ledger yet.{" "}
          <Link href="/decisions" style={{ color: "var(--text-secondary)" }}>View Decision Feed →</Link>
        </div>
      )}

      {/* Governance footer */}
      <div style={{ marginTop: "16px", padding: "14px 16px", background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "8px" }}>
        <div style={{ fontSize: "11px", color: "var(--text-muted)", lineHeight: "1.6" }}>
          <strong style={{ color: "var(--text-secondary)" }}>Governance:</strong>{" "}
          Recommendations are derived from frozen HDV-001 historical evidence (728 COMPLETE decisions).
          Target-before-risk rates describe what happened in comparable past decisions — they are{" "}
          <strong>not forward probabilities of success</strong>.
          Policy version v0 is frozen with HDV-001. HDV-002 opens 2026-08-18 (≥200 COMPLETE decisions required for v1).
        </div>
      </div>
    </div>
  );
}
