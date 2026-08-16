import { promises as fs } from "fs";
import path from "path";
import type { PE2Ledger, ExecutionReport } from "@/lib/data";
import { formatDateTime } from "@/lib/data";

async function getData() {
  const dataDir = path.join(process.cwd(), "public", "data");
  const [pe2Raw, execRaw] = await Promise.all([
    fs.readFile(path.join(dataDir, "pe2_replay_ledger.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "execution_report.json"), "utf-8"),
  ]);
  return {
    pe2: JSON.parse(pe2Raw) as PE2Ledger,
    exec: JSON.parse(execRaw) as ExecutionReport,
  };
}

export default async function AuditPage() {
  const { pe2, exec } = await getData();

  const checks = [
    {
      id: "determinism",
      label: "Determinism",
      pass: pe2.determinism_pass,
      description: "Same certified state at T + same frozen policy = same sealed decision. The engine produces identical output on repeated runs.",
      detail: "Verified across P.E.2 historical lifecycle replay. 7 instruments, 7 decisions, 7 execution intents.",
    },
    {
      id: "lookahead",
      label: "No-Lookahead",
      pass: pe2.lookahead_clean,
      description: "Future information was not available to the decision engine at T. The certified state contains only data that existed at the decision session.",
      detail: `peeked_returns_at_seal: ${pe2.peeked_returns_at_seal}. No future price data was accessible during decision formation.`,
    },
    {
      id: "poison",
      label: "Poison Test",
      pass: pe2.poison_test_pass,
      description: "A deliberate attempt to inject future data into the certified state was rejected. The system correctly identified and blocked the contamination.",
      detail: "Poison test passed. The temporal boundary is enforced at the data layer, not just by convention.",
    },
    {
      id: "immutable",
      label: "Immutable Seal",
      pass: !pe2.prospective_cohort_mutated,
      description: "Once a decision is sealed, it cannot be modified. The prospective cohort was not mutated during or after the replay.",
      detail: `prospective_cohort_mutated: ${pe2.prospective_cohort_mutated}. Original decisions remain unchanged.`,
    },
    {
      id: "append_only",
      label: "Append-Only Evidence",
      pass: !pe2.protected_artifacts_mutated,
      description: "Evidence is appended to the ledger after the fact. Protected artifacts (decisions, intents, policy) were not modified.",
      detail: `protected_artifacts_mutated: ${pe2.protected_artifacts_mutated}. Evidence cannot retroactively alter the original decision.`,
    },
    {
      id: "statistical_backtest",
      label: "Not a Statistical Backtest",
      pass: !pe2.statistical_backtest,
      description: "This replay is a lifecycle demonstration, not a statistical backtest. No parameter optimization was performed on the replay data.",
      detail: `statistical_backtest: ${pe2.statistical_backtest}. The +5% target and 20-session horizon are fixed control parameters.`,
    },
    {
      id: "pe1_lookahead",
      label: "P.E.1 No-Lookahead",
      pass: !exec.peeked_returns_at_seal,
      description: "The P.E.1 targeted execution replay also confirmed no lookahead. Returns were not peeked at seal time.",
      detail: `peeked_returns_at_seal: ${exec.peeked_returns_at_seal}. 14 execution intents verified.`,
    },
    {
      id: "pe1_cohort",
      label: "P.E.1 Cohort Integrity",
      pass: !exec.prospective_cohort_mutated,
      description: "The prospective cohort was not mutated during the P.E.1 targeted execution replay.",
      detail: `prospective_cohort_mutated: ${exec.prospective_cohort_mutated}.`,
    },
  ];

  const allPass = checks.every((c) => c.pass);

  return (
    <div style={{ maxWidth: "900px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Header */}
      <div style={{ marginBottom: "32px" }}>
        <p style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>Observatory</p>
        <h1 style={{ fontSize: "24px", fontWeight: "700", color: "var(--text-primary)", letterSpacing: "-0.02em", margin: "0 0 8px 0" }}>Audit Trail</h1>
        <p style={{ fontSize: "13px", color: "var(--text-secondary)", margin: 0 }}>
          Engineering evidence that the system prevents hindsight contamination.
        </p>
      </div>

      {/* Overall status */}
      <div
        style={{
          background: allPass ? "rgba(16,185,129,0.06)" : "rgba(239,68,68,0.06)",
          border: `1px solid ${allPass ? "rgba(16,185,129,0.2)" : "rgba(239,68,68,0.2)"}`,
          borderRadius: "12px",
          padding: "20px 24px",
          marginBottom: "32px",
          display: "flex",
          alignItems: "center",
          gap: "16px",
        }}
      >
        <div
          style={{
            width: "40px",
            height: "40px",
            borderRadius: "50%",
            background: allPass ? "rgba(16,185,129,0.15)" : "rgba(239,68,68,0.15)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: "18px",
            flexShrink: 0,
          }}
        >
          {allPass ? "✓" : "✗"}
        </div>
        <div>
          <div style={{ fontSize: "16px", fontWeight: "700", color: allPass ? "#10b981" : "#ef4444" }}>
            {allPass ? "All integrity checks pass" : "One or more checks failed"}
          </div>
          <div style={{ fontSize: "12px", color: "var(--text-secondary)", marginTop: "2px" }}>
            {checks.filter((c) => c.pass).length} / {checks.length} checks passed · P.E.2 lifecycle validation: {pe2.lifecycle_validation}
          </div>
        </div>
      </div>

      {/* Core principle */}
      <div
        style={{
          background: "var(--bg-card)",
          border: "1px solid rgba(59,130,246,0.2)",
          borderRadius: "12px",
          padding: "20px 24px",
          marginBottom: "32px",
          position: "relative",
          overflow: "hidden",
        }}
      >
        <div style={{ position: "absolute", top: 0, left: 0, right: 0, height: "2px", background: "linear-gradient(90deg, #3b82f6, #8b5cf6)" }} />
        <h2 style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 12px 0" }}>Core Temporal Guarantee</h2>
        <div style={{ fontSize: "14px", color: "var(--text-primary)", lineHeight: "1.7", fontWeight: "500" }}>
          Future data cannot modify a sealed decision.
        </div>
        <div style={{ fontSize: "12px", color: "var(--text-secondary)", marginTop: "8px", lineHeight: "1.6" }}>
          The system enforces a strict temporal boundary at T. Everything before T is the certified state. Everything after T is evidence. The decision is sealed at T and cannot be retroactively altered by what happens afterward.
        </div>
      </div>

      {/* Checks */}
      <div style={{ display: "flex", flexDirection: "column", gap: "12px", marginBottom: "32px" }}>
        {checks.map((check) => (
          <div
            key={check.id}
            style={{
              background: "var(--bg-card)",
              border: `1px solid ${check.pass ? "var(--border)" : "rgba(239,68,68,0.2)"}`,
              borderRadius: "10px",
              padding: "16px 20px",
            }}
          >
            <div style={{ display: "flex", alignItems: "flex-start", gap: "12px" }}>
              <div
                style={{
                  width: "20px",
                  height: "20px",
                  borderRadius: "50%",
                  background: check.pass ? "rgba(16,185,129,0.15)" : "rgba(239,68,68,0.15)",
                  border: `1px solid ${check.pass ? "rgba(16,185,129,0.3)" : "rgba(239,68,68,0.3)"}`,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: "10px",
                  fontWeight: "700",
                  color: check.pass ? "#10b981" : "#ef4444",
                  flexShrink: 0,
                  marginTop: "1px",
                }}
              >
                {check.pass ? "✓" : "✗"}
              </div>
              <div style={{ flex: 1 }}>
                <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "6px" }}>
                  <span style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)" }}>{check.label}</span>
                  <span
                    style={{
                      fontSize: "10px",
                      fontWeight: "700",
                      color: check.pass ? "#10b981" : "#ef4444",
                      background: check.pass ? "rgba(16,185,129,0.1)" : "rgba(239,68,68,0.1)",
                      padding: "1px 6px",
                      borderRadius: "3px",
                      letterSpacing: "0.05em",
                    }}
                  >
                    {check.pass ? "PASS" : "FAIL"}
                  </span>
                </div>
                <div style={{ fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.5", marginBottom: "6px" }}>{check.description}</div>
                <div style={{ fontSize: "11px", color: "var(--text-muted)", fontFamily: "monospace" }}>{check.detail}</div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* What has NOT been proven */}
      <div
        style={{
          background: "var(--bg-card)",
          border: "1px solid rgba(245,158,11,0.2)",
          borderRadius: "12px",
          padding: "20px 24px",
          marginBottom: "24px",
        }}
      >
        <h2 style={{ fontSize: "13px", fontWeight: "600", color: "#f59e0b", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 12px 0" }}>
          What Has NOT Been Proven
        </h2>
        <div style={{ fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.7" }}>
          The integrity checks above confirm the system&apos;s temporal mechanics. They do not establish:
        </div>
        <div style={{ marginTop: "12px", display: "flex", flexDirection: "column", gap: "6px" }}>
          {[
            "Statistical significance of the strategy",
            "Alpha or predictive superiority over a benchmark",
            "Optimality of the +5% target or 20-session horizon",
            "Robustness across a sufficiently large holdout",
            "Live profitability",
          ].map((item) => (
            <div key={item} style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <div style={{ width: "4px", height: "4px", borderRadius: "50%", background: "#f59e0b", flexShrink: 0 }} />
              <span style={{ fontSize: "12px", color: "var(--text-muted)" }}>{item}</span>
            </div>
          ))}
        </div>
        <div style={{ marginTop: "12px", fontSize: "11px", color: "var(--text-muted)", fontStyle: "italic" }}>
          Statistical validation (CS-P-007) is a future research gate. It is not a prerequisite for the current product demonstration.
        </div>
      </div>

      {/* Replay metadata */}
      <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "12px", padding: "20px 24px" }}>
        <h2 style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 16px 0" }}>Replay Metadata</h2>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
          {[
            { label: "P.E.2 Certified T", value: formatDateTime(pe2.certified_t) },
            { label: "P.E.2 Execution Contract", value: pe2.execution_contract_label },
            { label: "P.E.2 Target", value: `${(pe2.target_pct * 100).toFixed(1)}%` },
            { label: "P.E.2 Max Hold", value: `${pe2.max_holding_sessions} market sessions` },
            { label: "P.E.1 Contract", value: exec.execution_contract },
            { label: "P.E.1 Target Source", value: exec.target_source },
            { label: "P.E.1 Stop Exit", value: exec.stop_exit_authorized ? "Authorized" : "Not authorized" },
            { label: "P.E.1 Path Optimization", value: exec.target_path_optimization_authorized ? "Authorized" : "Not authorized" },
          ].map((row) => (
            <div key={row.label} style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", padding: "8px 0", borderBottom: "1px solid var(--border-subtle)" }}>
              <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>{row.label}</span>
              <span style={{ fontSize: "11px", color: "var(--text-primary)", fontWeight: "500", textAlign: "right", maxWidth: "240px" }}>{row.value}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}