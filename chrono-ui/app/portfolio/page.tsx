"use client";

/**
 * /portfolio — Portfolio Recommendations (Product MVP v0.1)
 *
 * User flow:
 *   1. Enter weekly investment amount, risk tolerance, investment horizon.
 *   2. Enter current holdings (instrument, qty, avg cost, current value).
 *   3. Submit → calls POST /api/portfolio/recommendations.
 *   4. View personalised ADD / HOLD / NO_ACTION recommendations.
 *
 * v0.1: certified decisions are embedded in this page.
 * v0.2: decisions will be fetched from the backend decision source.
 */

import { CSSProperties, useState } from "react";

// ─── Types ────────────────────────────────────────────────────────────────────

interface Holding {
  instrument: string;
  quantity: number;
  average_cost_inr: number;
  current_value_inr: number;
}

interface Recommendation {
  instrument: string;
  action: string;
  allocation_inr: number;
  c3_002_direction: string;
  entry_price: number;
  target_pct: number;
  target_price: number;
  risk_pct: number;
  risk_boundary: number;
  maximum_hold_sessions: number;
  rationale: string;
  decision_id: string;
  execution_intent_id: string;
  allocation_engine_version: string;
}

interface RecommendationsResponse {
  recommendations: Recommendation[];
  engine_version: string;
  as_of: string;
}

// ─── Style constants ──────────────────────────────────────────────────────────

const labelStyle: CSSProperties = {
  display: "block",
  fontSize: "11px",
  fontWeight: "600",
  color: "var(--text-muted)",
  textTransform: "uppercase",
  letterSpacing: "0.06em",
  marginBottom: "4px",
  marginTop: "12px",
};

const inputStyle: CSSProperties = {
  width: "100%",
  padding: "8px 10px",
  borderRadius: "6px",
  background: "var(--bg-secondary)",
  border: "1px solid var(--border)",
  color: "var(--text-primary)",
  fontSize: "13px",
  outline: "none",
  marginBottom: "4px",
};

const addBtnStyle: CSSProperties = {
  padding: "7px 12px",
  borderRadius: "6px",
  background: "rgba(59,130,246,0.12)",
  border: "1px solid rgba(59,130,246,0.25)",
  color: "#3b82f6",
  fontSize: "12px",
  fontWeight: "600",
  cursor: "pointer",
  width: "100%",
};

const thStyle: CSSProperties = {
  padding: "10px 14px",
  textAlign: "left",
  fontSize: "10px",
  fontWeight: "700",
  color: "var(--text-muted)",
  textTransform: "uppercase",
  letterSpacing: "0.08em",
  background: "var(--bg-secondary)",
};

const tdStyle: CSSProperties = {
  padding: "12px 14px",
  fontSize: "13px",
  color: "var(--text-secondary)",
  verticalAlign: "middle",
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

function actionColor(action: string): string {
  if (action === "Add") return "#10b981";
  if (action === "Hold") return "#f59e0b";
  if (action === "Avoid") return "#ef4444";
  return "#6b7280";
}

function actionLabel(action: string): string {
  if (action === "Add") return "ADD";
  if (action === "Hold") return "HOLD";
  if (action === "Avoid") return "AVOID";
  if (action === "NoAction") return "NO ACTION";
  return action.toUpperCase();
}

function directionColor(dir: string): string {
  if (dir === "LONG") return "#10b981";
  if (dir === "SHORT") return "#ef4444";
  return "#6b7280";
}

function fmt(n: number): string {
  return n.toLocaleString("en-IN", { maximumFractionDigits: 0 });
}

function pct(n: number): string {
  return `${(n * 100).toFixed(1)}%`;
}

// ─── Default holdings (demo) ──────────────────────────────────────────────────

const DEFAULT_HOLDINGS: Holding[] = [
  { instrument: "INFY.NS", quantity: 10, average_cost_inr: 1450, current_value_inr: 16200 },
  { instrument: "TCS.NS", quantity: 3, average_cost_inr: 3200, current_value_inr: 10440 },
];

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function PortfolioPage() {
  const [weeklyBudget, setWeeklyBudget] = useState(5000);
  const [riskTolerance, setRiskTolerance] = useState("Moderate");
  const [horizon, setHorizon] = useState("MediumTerm");
  const [holdings, setHoldings] = useState<Holding[]>(DEFAULT_HOLDINGS);
  const [newHolding, setNewHolding] = useState<Holding>({
    instrument: "", quantity: 0, average_cost_inr: 0, current_value_inr: 0,
  });
  const [result, setResult] = useState<RecommendationsResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function addHolding() {
    if (!newHolding.instrument.trim()) return;
    setHoldings((prev) => [...prev, { ...newHolding }]);
    setNewHolding({ instrument: "", quantity: 0, average_cost_inr: 0, current_value_inr: 0 });
  }

  function removeHolding(idx: number) {
    setHoldings((prev) => prev.filter((_, i) => i !== idx));
  }

  async function submit() {
    setLoading(true);
    setError(null);
    setResult(null);

    const existing_exposure_inr: Record<string, number> = {};
    for (const h of holdings) {
      existing_exposure_inr[h.instrument] =
        (existing_exposure_inr[h.instrument] ?? 0) + h.current_value_inr;
    }

    const body = {
      user_profile: {
        user_id: "demo-user",
        weekly_investment_inr: weeklyBudget,
        risk_tolerance: riskTolerance,
        investment_horizon: horizon,
      },
      portfolio: {
        as_of: new Date().toISOString(),
        available_cash_inr: weeklyBudget,
        holdings,
        existing_exposure_inr,
      },
      // v0.2: decisions are no longer sent by the client.
      // The backend fetches certified C3-002 + Coralys v0 decisions from its own source.
    };

    try {
      const res = await fetch("/api/portfolio/recommendations", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      if (!res.ok) {
        setError(data.error ?? "Unknown error from backend.");
      } else {
        setResult(data as RecommendationsResponse);
      }
    } catch (e) {
      setError(`Network error: ${String(e)}`);
    } finally {
      setLoading(false);
    }
  }

  const totalAllocated = result
    ? result.recommendations.reduce((s, r) => s + r.allocation_inr, 0)
    : 0;

  const addCount = result
    ? result.recommendations.filter((r) => r.action === "Add").length
    : 0;

  return (
    <div style={{ maxWidth: "1100px", margin: "0 auto", padding: "32px 24px" }}>

      {/* Header */}
      <div style={{ marginBottom: "32px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "8px" }}>
          <div style={{
            padding: "4px 10px", borderRadius: "4px",
            background: "rgba(16,185,129,0.1)", border: "1px solid rgba(16,185,129,0.25)",
            fontSize: "11px", fontWeight: "700", color: "#10b981", letterSpacing: "0.08em",
            textTransform: "uppercase",
          }}>
            Product MVP v0.1
          </div>
          <div style={{
            padding: "4px 10px", borderRadius: "4px",
            background: "rgba(59,130,246,0.1)", border: "1px solid rgba(59,130,246,0.25)",
            fontSize: "11px", fontWeight: "600", color: "#3b82f6",
          }}>
            C3-002 + Coralys v0 · Frozen 2026-08-16
          </div>
        </div>
        <h1 style={{ fontSize: "24px", fontWeight: "700", color: "var(--text-primary)", margin: 0 }}>
          Your Weekly Investment Plan
        </h1>
        <p style={{ color: "var(--text-secondary)", marginTop: "6px", fontSize: "14px", maxWidth: "600px" }}>
          Based on certified C3-002 decisions and sealed Coralys execution parameters.
          The system tells you what to do — your portfolio and budget determine how much.
        </p>
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "340px 1fr", gap: "24px", alignItems: "start" }}>

        {/* ── Left panel: inputs ── */}
        <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>

          {/* Profile card */}
          <div style={{
            background: "var(--bg-card)", border: "1px solid var(--border)",
            borderRadius: "10px", padding: "20px",
          }}>
            <h2 style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-secondary)", textTransform: "uppercase", letterSpacing: "0.08em", margin: "0 0 4px" }}>
              Your Profile
            </h2>

            <label style={labelStyle}>Weekly investment (₹)</label>
            <input
              type="number"
              value={weeklyBudget}
              onChange={(e) => setWeeklyBudget(Number(e.target.value))}
              style={inputStyle}
              min={100}
              step={500}
            />

            <label style={labelStyle}>Risk tolerance</label>
            <select value={riskTolerance} onChange={(e) => setRiskTolerance(e.target.value)} style={inputStyle}>
              <option value="Conservative">Conservative (50% of budget per instrument)</option>
              <option value="Moderate">Moderate (75% of budget per instrument)</option>
              <option value="Aggressive">Aggressive (100% of budget per instrument)</option>
            </select>

            <label style={labelStyle}>Investment horizon</label>
            <select value={horizon} onChange={(e) => setHorizon(e.target.value)} style={inputStyle}>
              <option value="ShortTerm">Short term (≤5 sessions)</option>
              <option value="MediumTerm">Medium term (≤20 sessions)</option>
              <option value="LongTerm">Long term (≤60 sessions)</option>
            </select>
          </div>

          {/* Holdings card */}
          <div style={{
            background: "var(--bg-card)", border: "1px solid var(--border)",
            borderRadius: "10px", padding: "20px",
          }}>
            <h2 style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-secondary)", textTransform: "uppercase", letterSpacing: "0.08em", margin: "0 0 12px" }}>
              Current Holdings
            </h2>

            {holdings.length === 0 && (
              <p style={{ color: "var(--text-muted)", fontSize: "13px", marginBottom: "12px" }}>
                No holdings. All instruments treated as new positions.
              </p>
            )}

            {holdings.map((h, i) => (
              <div key={i} style={{
                display: "flex", justifyContent: "space-between", alignItems: "center",
                padding: "8px 10px", borderRadius: "6px",
                background: "var(--bg-secondary)", marginBottom: "6px",
              }}>
                <div>
                  <div style={{ fontWeight: "600", color: "var(--text-primary)", fontSize: "13px" }}>{h.instrument}</div>
                  <div style={{ color: "var(--text-muted)", fontSize: "11px" }}>
                    {h.quantity} × ₹{fmt(h.average_cost_inr)} · Value ₹{fmt(h.current_value_inr)}
                  </div>
                </div>
                <button
                  onClick={() => removeHolding(i)}
                  style={{ background: "none", border: "none", color: "var(--text-muted)", cursor: "pointer", fontSize: "18px", padding: "0 4px", lineHeight: 1 }}
                >
                  ×
                </button>
              </div>
            ))}

            <div style={{ marginTop: "12px", display: "flex", flexDirection: "column", gap: "6px" }}>
              <input
                placeholder="Instrument (e.g. INFY.NS)"
                value={newHolding.instrument}
                onChange={(e) => setNewHolding((p) => ({ ...p, instrument: e.target.value }))}
                style={{ ...inputStyle, marginBottom: 0 }}
              />
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "6px" }}>
                <input
                  type="number"
                  placeholder="Qty"
                  value={newHolding.quantity || ""}
                  onChange={(e) => setNewHolding((p) => ({ ...p, quantity: Number(e.target.value) }))}
                  style={{ ...inputStyle, marginBottom: 0 }}
                />
                <input
                  type="number"
                  placeholder="Avg ₹"
                  value={newHolding.average_cost_inr || ""}
                  onChange={(e) => setNewHolding((p) => ({ ...p, average_cost_inr: Number(e.target.value) }))}
                  style={{ ...inputStyle, marginBottom: 0 }}
                />
                <input
                  type="number"
                  placeholder="Value ₹"
                  value={newHolding.current_value_inr || ""}
                  onChange={(e) => setNewHolding((p) => ({ ...p, current_value_inr: Number(e.target.value) }))}
                  style={{ ...inputStyle, marginBottom: 0 }}
                />
              </div>
              <button onClick={addHolding} style={addBtnStyle}>+ Add holding</button>
            </div>
          </div>

          {/* Submit */}
          <button
            onClick={submit}
            disabled={loading}
            style={{
              padding: "14px 20px",
              borderRadius: "8px",
              background: loading ? "rgba(59,130,246,0.4)" : "#3b82f6",
              border: "none",
              color: "white",
              fontSize: "14px",
              fontWeight: "700",
              cursor: loading ? "not-allowed" : "pointer",
              transition: "background 0.15s",
            }}
          >
            {loading ? "Computing…" : "Get my recommendations →"}
          </button>

          {error && (
            <div style={{
              padding: "12px 14px", borderRadius: "8px",
              background: "rgba(239,68,68,0.1)", border: "1px solid rgba(239,68,68,0.25)",
              color: "#ef4444", fontSize: "13px",
            }}>
              {error}
            </div>
          )}
        </div>

        {/* ── Right panel: results ── */}
        <div>
          {!result && !loading && (
            <div style={{
              background: "var(--bg-card)", border: "1px solid var(--border)",
              borderRadius: "10px", padding: "48px 32px", textAlign: "center",
            }}>
              <div style={{ fontSize: "32px", marginBottom: "12px" }}>📊</div>
              <div style={{ color: "var(--text-secondary)", fontSize: "14px" }}>
                Enter your profile and holdings, then click{" "}
                <strong style={{ color: "var(--text-primary)" }}>Get my recommendations</strong>.
              </div>
              <div style={{ color: "var(--text-muted)", fontSize: "12px", marginTop: "8px" }}>
                Decisions certified by C3-002 · Execution by Coralys v0 · Allocation by AllocationEngine v0
              </div>
            </div>
          )}

          {loading && (
            <div style={{
              background: "var(--bg-card)", border: "1px solid var(--border)",
              borderRadius: "10px", padding: "48px 32px", textAlign: "center",
              color: "var(--text-secondary)", fontSize: "14px",
            }}>
              Computing personalised recommendations…
            </div>
          )}

          {result && (
            <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>

              {/* Summary bar */}
              <div style={{
                background: "var(--bg-card)", border: "1px solid var(--border)",
                borderRadius: "10px", padding: "16px 20px",
                display: "flex", gap: "32px", alignItems: "center", flexWrap: "wrap",
              }}>
                <div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.08em" }}>Weekly budget</div>
                  <div style={{ fontSize: "20px", fontWeight: "700", color: "var(--text-primary)" }}>₹{fmt(weeklyBudget)}</div>
                </div>
                <div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.08em" }}>Allocated this week</div>
                  <div style={{ fontSize: "20px", fontWeight: "700", color: "#10b981" }}>₹{fmt(totalAllocated)}</div>
                </div>
                <div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.08em" }}>Unallocated</div>
                  <div style={{ fontSize: "20px", fontWeight: "700", color: "var(--text-secondary)" }}>₹{fmt(weeklyBudget - totalAllocated)}</div>
                </div>
                <div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.08em" }}>ADD signals</div>
                  <div style={{ fontSize: "20px", fontWeight: "700", color: "#10b981" }}>{addCount}</div>
                </div>
              </div>

              {/* Recommendations table */}
              <div style={{
                background: "var(--bg-card)", border: "1px solid var(--border)",
                borderRadius: "10px", overflow: "hidden",
              }}>
                <div style={{ padding: "16px 20px", borderBottom: "1px solid var(--border)" }}>
                  <h2 style={{ margin: 0, fontSize: "14px", fontWeight: "700", color: "var(--text-primary)" }}>
                    Recommendations
                  </h2>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                    {result.engine_version} · {new Date(result.as_of).toLocaleString("en-IN")}
                  </div>
                </div>

                <table style={{ width: "100%", borderCollapse: "collapse" }}>
                  <thead>
                    <tr style={{ borderBottom: "1px solid var(--border)" }}>
                      {["Instrument", "Signal", "Action", "Allocate this week", "Target", "Risk", "Max hold"].map((h) => (
                        <th key={h} style={thStyle}>{h}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {result.recommendations.map((rec, i) => (
                      <tr
                        key={i}
                        style={{
                          borderBottom: "1px solid var(--border-subtle)",
                          background: i % 2 === 0 ? "transparent" : "rgba(255,255,255,0.01)",
                        }}
                      >
                        <td style={tdStyle}>
                          <div style={{ fontWeight: "600", color: "var(--text-primary)", fontSize: "13px" }}>
                            {rec.instrument.replace(".NS", "")}
                          </div>
                          <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>{rec.instrument}</div>
                        </td>
                        <td style={tdStyle}>
                          <span style={{
                            padding: "2px 8px", borderRadius: "4px", fontSize: "11px", fontWeight: "700",
                            color: directionColor(rec.c3_002_direction),
                            background: `${directionColor(rec.c3_002_direction)}18`,
                          }}>
                            {rec.c3_002_direction}
                          </span>
                        </td>
                        <td style={tdStyle}>
                          <span style={{
                            padding: "3px 10px", borderRadius: "5px", fontSize: "12px", fontWeight: "700",
                            color: actionColor(rec.action),
                            background: `${actionColor(rec.action)}18`,
                            border: `1px solid ${actionColor(rec.action)}30`,
                          }}>
                            {actionLabel(rec.action)}
                          </span>
                        </td>
                        <td style={{ ...tdStyle, textAlign: "right" }}>
                          {rec.action === "Add" ? (
                            <div>
                              <div style={{ fontWeight: "700", color: "#10b981", fontSize: "14px" }}>
                                ₹{fmt(rec.allocation_inr)}
                              </div>
                              {rec.allocation_inr === 0 && (
                                <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>deferred</div>
                              )}
                            </div>
                          ) : (
                            <span style={{ color: "var(--text-muted)", fontSize: "13px" }}>—</span>
                          )}
                        </td>
                        <td style={{ ...tdStyle, textAlign: "right" }}>
                          <span style={{ color: "#10b981", fontSize: "13px", fontWeight: "600" }}>
                            +{pct(rec.target_pct)}
                          </span>
                        </td>
                        <td style={{ ...tdStyle, textAlign: "right" }}>
                          <span style={{ color: "#ef4444", fontSize: "13px", fontWeight: "600" }}>
                            −{pct(rec.risk_pct)}
                          </span>
                        </td>
                        <td style={{ ...tdStyle, textAlign: "right" }}>
                          <span style={{ color: "var(--text-secondary)", fontSize: "13px" }}>
                            {rec.maximum_hold_sessions}s
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {/* Provenance footer */}
              <div style={{
                background: "var(--bg-card)", border: "1px solid var(--border)",
                borderRadius: "10px", padding: "14px 20px",
                display: "flex", gap: "24px", flexWrap: "wrap",
              }}>
                <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                  <span style={{ color: "var(--text-secondary)", fontWeight: "600" }}>Decision source:</span>{" "}
                  C3-002 (sealed)
                </div>
                <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                  <span style={{ color: "var(--text-secondary)", fontWeight: "600" }}>Execution:</span>{" "}
                  coralys-exec-v0 · 3876ffa2
                </div>
                <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                  <span style={{ color: "var(--text-secondary)", fontWeight: "600" }}>Allocation:</span>{" "}
                  {result.recommendations[0]?.allocation_engine_version ?? "allocation-engine-v0"}
                </div>
                <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                  <span style={{ color: "var(--text-secondary)", fontWeight: "600" }}>Engine:</span>{" "}
                  {result.engine_version}
                </div>
              </div>

              {/* Explanation note */}
              <div style={{
                padding: "12px 16px", borderRadius: "8px",
                background: "rgba(59,130,246,0.06)", border: "1px solid rgba(59,130,246,0.15)",
                fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.6",
              }}>
                <strong style={{ color: "var(--text-primary)" }}>How to read this:</strong>{" "}
                <strong>ADD</strong> means the certified decision supports increasing this position.
                The allocation amount reflects your weekly budget and existing exposure.
                If allocation shows ₹0, the signal is still valid — allocate when cash is available.{" "}
                <strong>HOLD</strong> means maintain your existing position; do not add this week.{" "}
                <strong>NO ACTION</strong> means no certified signal for this instrument at the current session.
              </div>

            </div>
          )}
        </div>
      </div>
    </div>
  );
}