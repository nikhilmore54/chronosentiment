"use client";

/**
 * /live — Trading Home Screen (v1)
 *
 * Fetches the ranked RecommendationSnapshotV1 from GET /recommendations/v1/latest
 * (coralys_decision_server :3001) and displays it as the primary trading
 * home screen.
 *
 * Architecture contract:
 * - ALL recommendation logic lives in the Rust engine (coralys-decision).
 * - This page displays RecommendationRecordV1 fields verbatim — no re-ranking,
 *   no re-scoring, no probability claims.
 * - Historical target-before-risk rates are NOT presented as forward
 *   probabilities of success.
 * - Adaptive geometry is derived from first-exit analogue population:
 *   target from winning analogues, risk from losing analogues.
 */

import { useEffect, useState } from "react";
import Link from "next/link";

// ─── Types (mirror RecommendationRecordV1 from the Rust engine) ───────────────

interface RecommendationRecordV1 {
  decision_id: string;
  instrument: string;
  direction: string;
  trend: string;
  momentum: string;
  reference_price: number | null;
  adaptive_target: number | null;
  adaptive_risk: number | null;
  adaptive_upside_pct: number | null;
  adaptive_downside_pct: number | null;
  adaptive_rr: number | null;
  adaptive_horizon_sessions: number | null;
  degradation_level: string;
  sample_size: number;
  target_rate: number;
  evidence_class: "Favourable" | "Mixed" | "Unfavourable" | "Insufficient";
  action: "Buy" | "Watch" | "NoTrade";
  rank_score: number;
  recommendation_policy_version: string;
  vol_regime: string;
  volume_regime: string;
}

interface RecommendationSnapshotV1 {
  evaluated: number;
  actionable: number;
  buy: number;
  watch: number;
  no_trade: number;
  policy_version: string;
  recommendations: RecommendationRecordV1[];
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

function fmtPctDirect(n: number | null): string {
  // For values already in percent (e.g. adaptive_target_pct stored as 3.1 meaning 3.1%)
  if (n === null || n === undefined) return "—";
  return n.toFixed(2) + "%";
}

function actionColor(action: RecommendationRecordV1["action"]): string {
  switch (action) {
    case "Buy": return "#10b981";
    case "Watch": return "#f59e0b";
    case "NoTrade": return "#6b7280";
  }
}

function actionLabel(action: RecommendationRecordV1["action"]): string {
  switch (action) {
    case "Buy": return "BUY";
    case "Watch": return "WATCH";
    case "NoTrade": return "NO TRADE";
  }
}

function evidenceColor(cls: RecommendationRecordV1["evidence_class"]): string {
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

function degradationBadge(level: string): { label: string; color: string } {
  switch (level) {
    case "Exact":        return { label: "Exact match",    color: "#10b981" };
    case "RelaxVolume":  return { label: "Relax vol",      color: "#6366f1" };
    case "RelaxBoth":    return { label: "Relax both",     color: "#f59e0b" };
    case "StateOnly":    return { label: "State only",     color: "#ef4444" };
    case "Insufficient": return { label: "Insufficient",   color: "#6b7280" };
    default:             return { label: level,            color: "#6b7280" };
  }
}

// ─── Recommendation Card (v1) ─────────────────────────────────────────────────

function RecommendationCard({ rec }: { rec: RecommendationRecordV1 }) {
  const ac = actionColor(rec.action);
  const ec = evidenceColor(rec.evidence_class);
  const isBuy = rec.action === "Buy";
  const isWatch = rec.action === "Watch";
  const deg = degradationBadge(rec.degradation_level);

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
            <span style={{ fontSize: "15px", fontWeight: "700", color: "var(--text-primary)" }}>
              {rec.instrument.replace("_NS", ".NS")}
            </span>
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

        {/* State row */}
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "10px", marginBottom: "12px" }}>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Coralys State</div>
            <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
              {rec.trend} / {rec.momentum}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Vol / Volume</div>
            <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
              {rec.vol_regime} / {rec.volume_regime}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Analogues</div>
            <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
              <span style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>n={rec.sample_size}</span>
              <span style={{ fontSize: "9px", fontWeight: "600", color: deg.color, background: `${deg.color}18`, padding: "1px 5px", borderRadius: "3px" }}>
                {deg.label}
              </span>
            </div>
          </div>
        </div>

        {/* Adaptive geometry row */}
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: "10px", marginBottom: "12px" }}>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Reference</div>
            <div style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-primary)" }}>
              {fmt(rec.reference_price)}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Adap. Target</div>
            <div style={{ fontSize: "13px", fontWeight: "700", color: "#10b981" }}>
              {fmt(rec.adaptive_target)}
              {rec.adaptive_upside_pct !== null && (
                <span style={{ fontSize: "10px", color: "#10b981", marginLeft: "4px" }}>
                  {rec.direction === "SHORT" ? "-" : "+"}{fmtPct(rec.adaptive_upside_pct)}
                </span>
              )}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Adap. Risk</div>
            <div style={{ fontSize: "13px", fontWeight: "700", color: "#ef4444" }}>
              {fmt(rec.adaptive_risk)}
              {rec.adaptive_downside_pct !== null && (
                <span style={{ fontSize: "10px", color: "#ef4444", marginLeft: "4px" }}>
                  {rec.direction === "SHORT" ? "+" : "-"}{fmtPct(rec.adaptive_downside_pct)}
                </span>
              )}
            </div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>R:R</div>
            <div style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-primary)" }}>
              {rec.adaptive_rr !== null ? rec.adaptive_rr.toFixed(2) : "—"}
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
              {rec.evidence_class.toUpperCase()} EVIDENCE
            </span>
            <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>
              Horizon: {rec.adaptive_horizon_sessions !== null ? rec.adaptive_horizon_sessions.toFixed(1) + " sessions" : "—"}
            </span>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px" }}>
            <div>
              <div style={{ fontSize: "9px", color: "var(--text-muted)", marginBottom: "1px" }}>Target rate (first-exit)</div>
              <div style={{ fontSize: "12px", fontWeight: "700", color: ec }}>
                {fmtPct(rec.target_rate)}
              </div>
            </div>
            <div>
              <div style={{ fontSize: "9px", color: "var(--text-muted)", marginBottom: "1px" }}>Rank score</div>
              <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
                {rec.rank_score.toFixed(4)}
              </div>
            </div>
          </div>
          <div style={{ marginTop: "6px", fontSize: "9px", color: "var(--text-muted)", fontStyle: "italic" }}>
            Historical rates describe past analogues — not forward probabilities of success.
          </div>
        </div>

        {/* Footer: policy + link */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>
            policy {rec.recommendation_policy_version}
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
  const [snapshot, setSnapshot] = useState<RecommendationSnapshotV1 | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const market = getMarketStatus();

  useEffect(() => {
    fetch("/api/recommendations/v1/latest")
      .then((r) => {
        if (!r.ok) throw new Error(`Server returned ${r.status}`);
        return r.json() as Promise<RecommendationSnapshotV1>;
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
            NSE · Ticker-specific analogue population · Adaptive geometry · Policy {snapshot?.policy_version ?? "v1"}
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
        <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: "12px", marginBottom: "28px" }}>
          {[
            { label: "Evaluated",  value: snapshot.evaluated,  color: "var(--text-secondary)" },
            { label: "Actionable", value: snapshot.actionable, color: "#10b981" },
            { label: "BUY",        value: snapshot.buy,        color: "#10b981" },
            { label: "WATCH",      value: snapshot.watch,      color: "#f59e0b" },
            { label: "NO TRADE",   value: snapshot.no_trade,   color: "#6b7280" },
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
          Loading v1 recommendations from Decision Server…
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
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(380px, 1fr))", gap: "12px" }}>
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
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(380px, 1fr))", gap: "12px" }}>
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
        <div style={{ fontSize: "10px", color: "var(--text-muted)", lineHeight: "1.6" }}>
          <strong style={{ color: "var(--text-secondary)" }}>Coralys v1 — Analogue-population engine.</strong>{" "}
          Adaptive target derived from 25th-percentile MFE of winning analogues (TARGET_BEFORE_RISK).
          Adaptive risk derived from median MAE of losing analogues (RISK_BEFORE_TARGET).
          Horizon from median sessions_to_outcome. Degradation: Exact → RelaxVolume → RelaxBoth → StateOnly → NO_TRADE.
          Historical rates describe past analogues — not forward probabilities of success.
          This system is for paper-trading observation only.
        </div>
      </div>
    </div>
  );
}
