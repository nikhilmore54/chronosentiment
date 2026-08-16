import { promises as fs } from "fs";
import path from "path";
import type { PE2Ledger, PE3Ledger } from "@/lib/data";
import { formatDate, formatReturn, shortHash } from "@/lib/data";

async function getData() {
  const dataDir = path.join(process.cwd(), "public", "data");
  const [pe2Raw, pe3Raw] = await Promise.all([
    fs.readFile(path.join(dataDir, "pe2_replay_ledger.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "pe3_replay_ledger.json"), "utf-8"),
  ]);
  return {
    pe2: JSON.parse(pe2Raw) as PE2Ledger,
    pe3: JSON.parse(pe3Raw) as PE3Ledger,
  };
}

const integrityChecks = (det: boolean, look: boolean, poison: boolean) => [
  { label: "Determinism", pass: det },
  { label: "No-lookahead", pass: look },
  { label: "Poison test", pass: poison },
];

function CheckBadges({ checks }: { checks: { label: string; pass: boolean }[] }) {
  return (
    <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
      {checks.map((c) => (
        <div
          key={c.label}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "4px",
            padding: "4px 10px",
            background: c.pass ? "rgba(16,185,129,0.08)" : "rgba(239,68,68,0.08)",
            border: `1px solid ${c.pass ? "rgba(16,185,129,0.15)" : "rgba(239,68,68,0.15)"}`,
            borderRadius: "4px",
          }}
        >
          <div
            style={{
              width: "5px",
              height: "5px",
              borderRadius: "50%",
              background: c.pass ? "#10b981" : "#ef4444",
            }}
          />
          <span style={{ fontSize: "11px", color: "var(--text-secondary)" }}>{c.label}</span>
          <span
            style={{
              fontSize: "11px",
              fontWeight: "700",
              color: c.pass ? "#10b981" : "#ef4444",
            }}
          >
            {c.pass ? "PASS" : "FAIL"}
          </span>
        </div>
      ))}
    </div>
  );
}

function ExitBar({
  n_target,
  n_risk,
  n_horizon,
  n_decisions,
}: {
  n_target: number;
  n_risk: number;
  n_horizon: number;
  n_decisions: number;
}) {
  const total = n_decisions || 1;
  return (
    <div style={{ display: "flex", gap: "16px", flexWrap: "wrap" }}>
      {[
        { label: "TARGET", n: n_target, color: "#10b981" },
        { label: "RISK", n: n_risk, color: "#ef4444" },
        { label: "HORIZON", n: n_horizon, color: "#6366f1" },
      ].map((s) => (
        <div key={s.label} style={{ display: "flex", flexDirection: "column", gap: "4px", minWidth: "80px" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
            <span style={{ fontSize: "11px", color: "var(--text-muted)", fontWeight: "600", letterSpacing: "0.06em" }}>
              {s.label}
            </span>
            <span style={{ fontSize: "13px", fontWeight: "700", color: s.color }}>
              {s.n}
            </span>
          </div>
          <div style={{ height: "4px", background: "var(--bg-hover)", borderRadius: "2px", overflow: "hidden" }}>
            <div
              style={{
                height: "100%",
                width: `${(s.n / total) * 100}%`,
                background: s.color,
                borderRadius: "2px",
              }}
            />
          </div>
          <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>
            {((s.n / total) * 100).toFixed(0)}%
          </span>
        </div>
      ))}
    </div>
  );
}

export default async function ReplayPage() {
  const { pe2, pe3 } = await getData();

  return (
    <div style={{ maxWidth: "1100px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Header */}
      <div style={{ marginBottom: "32px" }}>
        <p
          style={{
            fontSize: "11px",
            fontWeight: "600",
            color: "var(--text-muted)",
            letterSpacing: "0.1em",
            textTransform: "uppercase",
            marginBottom: "8px",
          }}
        >
          Observatory
        </p>
        <h1
          style={{
            fontSize: "24px",
            fontWeight: "700",
            color: "var(--text-primary)",
            letterSpacing: "-0.02em",
            margin: "0 0 8px 0",
          }}
        >
          Historical Replay
        </h1>
        <p
          style={{
            fontSize: "13px",
            color: "var(--text-secondary)",
            margin: 0,
            maxWidth: "560px",
          }}
        >
          The clock is moved back to the certified session. The engine only knows what was available at that time. Evidence is recorded afterward.
        </p>
      </div>

      {/* HISTORICAL label */}
      <div
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: "8px",
          padding: "6px 14px",
          background: "rgba(245,158,11,0.08)",
          border: "1px solid rgba(245,158,11,0.2)",
          borderRadius: "6px",
          marginBottom: "32px",
        }}
      >
        <div style={{ width: "6px", height: "6px", borderRadius: "50%", background: "#f59e0b" }} />
        <span
          style={{
            fontSize: "11px",
            fontWeight: "700",
            color: "#f59e0b",
            letterSpacing: "0.08em",
            textTransform: "uppercase",
          }}
        >
          Historical Replay — Not Live Performance
        </span>
      </div>

      {/* ── P.E.3 — CURRENT EXECUTION MODEL ─────────────────────────────────── */}
      <div
        style={{
          background: "var(--bg-card)",
          border: "1px solid rgba(99,102,241,0.35)",
          borderRadius: "12px",
          padding: "24px",
          marginBottom: "24px",
        }}
      >
        {/* Current badge */}
        <div
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "6px",
            padding: "3px 10px",
            background: "rgba(99,102,241,0.12)",
            border: "1px solid rgba(99,102,241,0.3)",
            borderRadius: "4px",
            marginBottom: "16px",
          }}
        >
          <div style={{ width: "5px", height: "5px", borderRadius: "50%", background: "#6366f1" }} />
          <span
            style={{
              fontSize: "10px",
              fontWeight: "700",
              color: "#6366f1",
              letterSpacing: "0.08em",
              textTransform: "uppercase",
            }}
          >
            Current Execution Model
          </span>
        </div>

        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: "16px",
            marginBottom: "24px",
          }}
        >
          <div>
            <h2
              style={{
                fontSize: "16px",
                fontWeight: "700",
                color: "var(--text-primary)",
                margin: "0 0 4px 0",
              }}
            >
              P.E.3 — coralys-exec-v0 (ATR/TMV)
            </h2>
            <p style={{ fontSize: "12px", color: "var(--text-secondary)", margin: 0 }}>
              Certified session: {formatDate(pe3.certified_t)} · {pe3.n_decisions} instruments ·{" "}
              {pe3.execution_contract_label}
            </p>
          </div>
          <CheckBadges
            checks={integrityChecks(pe3.determinism_pass, pe3.lookahead_clean, pe3.poison_test_pass)}
          />
        </div>

        {/* Artifact + eligibility */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
            gap: "12px",
            marginBottom: "24px",
          }}
        >
          {[
            { label: "Execution contract", value: pe3.execution_contract },
            { label: "Coralys artifact", value: shortHash(pe3.coralys_artifact_hash) },
            { label: "Model", value: `${pe3.coralys_model_id} v${pe3.coralys_model_version}` },
            { label: "Max hold", value: `${pe3.max_holding_sessions} sessions` },
            { label: "P.E.3 eligible", value: `${pe3.n_pe3_eligible} / ${pe3.n_decisions}` },
            { label: "ATR excluded", value: String(pe3.n_excluded_no_atr) },
          ].map((m) => (
            <div
              key={m.label}
              style={{
                background: "var(--bg-hover)",
                borderRadius: "8px",
                padding: "12px 14px",
              }}
            >
              <div style={{ fontSize: "10px", color: "var(--text-muted)", fontWeight: "600", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "4px" }}>
                {m.label}
              </div>
              <div style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)", fontFamily: "monospace" }}>
                {m.value}
              </div>
            </div>
          ))}
        </div>

        {/* Exit distribution */}
        <div style={{ marginBottom: "24px" }}>
          <p style={{ fontSize: "11px", color: "var(--text-muted)", fontWeight: "600", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "12px" }}>
            Exit distribution
          </p>
          <ExitBar
            n_target={pe3.n_target}
            n_risk={pe3.n_risk}
            n_horizon={pe3.n_horizon}
            n_decisions={pe3.n_pe3_eligible || pe3.n_decisions}
          />
        </div>

        {/* Per-record table */}
        <div>
          <p style={{ fontSize: "11px", color: "var(--text-muted)", fontWeight: "600", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "12px" }}>
            Execution records
          </p>
          <div style={{ overflowX: "auto" }}>
            <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "12px" }}>
              <thead>
                <tr>
                  {["Instrument", "Action", "Target %", "Risk %", "ATR(14)", "TMV state", "Exit", "Hold", "Return"].map((h) => (
                    <th
                      key={h}
                      style={{
                        textAlign: "left",
                        padding: "8px 10px",
                        color: "var(--text-muted)",
                        fontWeight: "600",
                        fontSize: "10px",
                        letterSpacing: "0.06em",
                        textTransform: "uppercase",
                        borderBottom: "1px solid var(--border)",
                      }}
                    >
                      {h}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {pe3.records.map((r) => {
                  const exitColor =
                    r.exit.exit_reason === "TARGET"
                      ? "#10b981"
                      : r.exit.exit_reason === "HORIZON"
                      ? "#6366f1"
                      : r.exit.exit_reason === "STOP"
                      ? "#ef4444"
                      : "var(--text-muted)";
                  return (
                    <tr
                      key={r.instrument}
                      style={{ borderBottom: "1px solid var(--border)" }}
                    >
                      <td style={{ padding: "10px 10px", fontWeight: "600", color: "var(--text-primary)" }}>
                        {r.instrument}
                      </td>
                      <td style={{ padding: "10px 10px", color: r.decision.action === "LONG" ? "#10b981" : r.decision.action === "SHORT" ? "#ef4444" : "var(--text-muted)" }}>
                        {r.decision.action}
                      </td>
                      <td style={{ padding: "10px 10px", color: "var(--text-primary)", fontFamily: "monospace" }}>
                        {r.coralys_target_pct != null ? `${(r.coralys_target_pct * 100).toFixed(2)}%` : "—"}
                      </td>
                      <td style={{ padding: "10px 10px", color: "var(--text-secondary)", fontFamily: "monospace" }}>
                        {r.coralys_risk_pct != null ? `${(r.coralys_risk_pct * 100).toFixed(2)}%` : "—"}
                      </td>
                      <td style={{ padding: "10px 10px", color: "var(--text-secondary)", fontFamily: "monospace" }}>
                        {r.atr_14_at_t != null ? r.atr_14_at_t.toFixed(2) : "—"}
                      </td>
                      <td style={{ padding: "10px 10px", color: "var(--text-secondary)" }}>
                        {r.coralys_tmv_state ?? "—"}
                      </td>
                      <td style={{ padding: "10px 10px", fontWeight: "700", color: exitColor }}>
                        {r.exit.exit_reason}
                      </td>
                      <td style={{ padding: "10px 10px", color: "var(--text-secondary)" }}>
                        {r.exit.holding_sessions != null
                          ? r.exit.holding_sessions === 1
                            ? "1 session"
                            : `${r.exit.holding_sessions} sessions`
                          : "—"}
                      </td>
                      <td style={{ padding: "10px 10px", fontFamily: "monospace", color: r.exit.decision_value != null && r.exit.decision_value > 0 ? "#10b981" : r.exit.decision_value != null && r.exit.decision_value < 0 ? "#ef4444" : "var(--text-muted)" }}>
                        {r.exit.decision_value != null ? formatReturn(r.exit.decision_value) : "—"}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* ── P.E.2 — ARCHIVED CONTROL ─────────────────────────────────────────── */}
      <div
        style={{
          background: "var(--bg-card)",
          border: "1px solid var(--border)",
          borderRadius: "12px",
          padding: "24px",
          marginBottom: "24px",
          opacity: 0.85,
        }}
      >
        {/* Archived badge */}
        <div
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "6px",
            padding: "3px 10px",
            background: "rgba(245,158,11,0.08)",
            border: "1px solid rgba(245,158,11,0.2)",
            borderRadius: "4px",
            marginBottom: "16px",
          }}
        >
          <div style={{ width: "5px", height: "5px", borderRadius: "50%", background: "#f59e0b" }} />
          <span
            style={{
              fontSize: "10px",
              fontWeight: "700",
              color: "#f59e0b",
              letterSpacing: "0.08em",
              textTransform: "uppercase",
            }}
          >
            Archived Control — IMMUTABLE
          </span>
        </div>

        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: "16px",
            marginBottom: "24px",
          }}
        >
          <div>
            <h2
              style={{
                fontSize: "16px",
                fontWeight: "700",
                color: "var(--text-primary)",
                margin: "0 0 4px 0",
              }}
            >
              P.E.2 — Fixed +5% Execution Contract
            </h2>
            <p style={{ fontSize: "12px", color: "var(--text-secondary)", margin: 0 }}>
              Certified session: {formatDate(pe2.certified_t)} · {pe2.n_decisions} instruments · Execution Contract v0 (fixed +5%, 20 sessions)
            </p>
          </div>
          <CheckBadges
            checks={integrityChecks(pe2.determinism_pass, pe2.lookahead_clean, pe2.poison_test_pass)}
          />
        </div>

        {/* Stats */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))",
            gap: "12px",
            marginBottom: "24px",
          }}
        >
          {[
            { label: "Target %", value: `${(pe2.target_pct * 100).toFixed(1)}%` },
            { label: "Max hold", value: `${pe2.max_holding_sessions} sessions` },
            { label: "TARGET exits", value: String(pe2.n_target) },
            { label: "HORIZON exits", value: String(pe2.n_horizon) },
            { label: "Gap-through", value: String(pe2.n_gap_through) },
            { label: "Peeked returns", value: pe2.peeked_returns_at_seal ? "YES" : "NO" },
          ].map((m) => (
            <div
              key={m.label}
              style={{
                background: "var(--bg-hover)",
                borderRadius: "8px",
                padding: "12px 14px",
              }}
            >
              <div style={{ fontSize: "10px", color: "var(--text-muted)", fontWeight: "600", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "4px" }}>
                {m.label}
              </div>
              <div style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)", fontFamily: "monospace" }}>
                {m.value}
              </div>
            </div>
          ))}
        </div>

        {/* Exit distribution */}
        <div style={{ marginBottom: "24px" }}>
          <p style={{ fontSize: "11px", color: "var(--text-muted)", fontWeight: "600", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "12px" }}>
            Exit distribution
          </p>
          <ExitBar
            n_target={pe2.n_target}
            n_risk={0}
            n_horizon={pe2.n_horizon}
            n_decisions={pe2.n_decisions}
          />
        </div>

        {/* Per-record table */}
        <div>
          <p style={{ fontSize: "11px", color: "var(--text-muted)", fontWeight: "600", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "12px" }}>
            Execution records
          </p>
          <div style={{ overflowX: "auto" }}>
            <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "12px" }}>
              <thead>
                <tr>
                  {["Instrument", "Action", "Target %", "Entry", "Exit reason", "Hold", "Return"].map((h) => (
                    <th
                      key={h}
                      style={{
                        textAlign: "left",
                        padding: "8px 10px",
                        color: "var(--text-muted)",
                        fontWeight: "600",
                        fontSize: "10px",
                        letterSpacing: "0.06em",
                        textTransform: "uppercase",
                        borderBottom: "1px solid var(--border)",
                      }}
                    >
                      {h}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {pe2.records.map((r) => {
                  const exitColor =
                    r.exit.exit_reason === "TARGET"
                      ? "#10b981"
                      : r.exit.exit_reason === "HORIZON"
                      ? "#6366f1"
                      : "var(--text-muted)";
                  return (
                    <tr
                      key={r.instrument}
                      style={{ borderBottom: "1px solid var(--border)" }}
                    >
                      <td style={{ padding: "10px 10px", fontWeight: "600", color: "var(--text-primary)" }}>
                        {r.instrument}
                      </td>
                      <td style={{ padding: "10px 10px", color: r.decision.action === "LONG" ? "#10b981" : r.decision.action === "SHORT" ? "#ef4444" : "var(--text-muted)" }}>
                        {r.decision.action}
                      </td>
                      <td style={{ padding: "10px 10px", color: "var(--text-primary)", fontFamily: "monospace" }}>
                        {(pe2.target_pct * 100).toFixed(1)}%
                      </td>
                      <td style={{ padding: "10px 10px", color: "var(--text-secondary)", fontFamily: "monospace" }}>
                        {r.intent.entry_price.toFixed(2)}
                      </td>
                      <td style={{ padding: "10px 10px", fontWeight: "700", color: exitColor }}>
                        {r.exit.exit_reason}
                      </td>
                      <td style={{ padding: "10px 10px", color: "var(--text-secondary)" }}>
                        {r.exit.holding_sessions != null ? `${r.exit.holding_sessions}s` : "—"}
                      </td>
                      <td style={{ padding: "10px 10px", fontFamily: "monospace", color: r.exit.decision_value != null && r.exit.decision_value > 0 ? "#10b981" : r.exit.decision_value != null && r.exit.decision_value < 0 ? "#ef4444" : "var(--text-muted)" }}>
                        {r.exit.decision_value != null ? formatReturn(r.exit.decision_value) : "—"}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Lineage note */}
      <div
        style={{
          background: "rgba(99,102,241,0.04)",
          border: "1px solid rgba(99,102,241,0.12)",
          borderRadius: "8px",
          padding: "16px 20px",
        }}
      >
        <p style={{ fontSize: "12px", color: "var(--text-secondary)", margin: 0, lineHeight: "1.6" }}>
          <strong style={{ color: "var(--text-primary)" }}>Execution lineage:</strong>{" "}
          P.E.2 (fixed +5%) is the archived control baseline. P.E.3 (coralys-exec-v0, ATR/TMV) is the current live execution model.
          Both replays use the same C3-002 direction decisions and the same historical market data.
          The only difference is the execution contract applied after the direction is sealed.
          Statistical comparison (CS-P-007) is a separate evidence track.
        </p>
      </div>
    </div>
  );
}