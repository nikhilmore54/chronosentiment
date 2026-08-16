import Link from "next/link";
import { promises as fs } from "fs";
import path from "path";
import type { HistoricalLedger, ExecutionReport, PE2Ledger } from "@/lib/data";
import { formatDate, shortHash, formatReturn, formatPrice } from "@/lib/data";

async function getData() {
  const dataDir = path.join(process.cwd(), "public", "data");
  const [histRaw, execRaw, pe2Raw, prospRaw] = await Promise.all([
    fs.readFile(path.join(dataDir, "historical_ledger.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "execution_report.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "pe2_replay_ledger.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "prospective_ledger.json"), "utf-8"),
  ]);
  return {
    hist: JSON.parse(histRaw) as HistoricalLedger,
    exec: JSON.parse(execRaw) as ExecutionReport,
    pe2: JSON.parse(pe2Raw) as PE2Ledger,
    prosp: JSON.parse(prospRaw) as HistoricalLedger,
  };
}

export default async function ObservatoryPage() {
  const { hist, exec, pe2, prosp } = await getData();

  const totalDecisions = hist.decisions.length + prosp.decisions.length;
  const completedObs = hist.observations.length;
  const openDecisions = prosp.decisions.length;
  const targetExits = exec.n_target;
  const horizonExits = exec.n_horizon;

  // Recent decisions from prospective (live cohort)
  const liveDecisions = prosp.decisions.slice(0, 7);

  // Recent completed from historical
  const recentCompleted = hist.decisions
    .filter((d) => hist.observations.find((o) => o.decision_id === d.decision_id))
    .slice(-5)
    .reverse();

  return (
    <div style={{ maxWidth: "1200px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Header */}
      <div style={{ marginBottom: "40px" }}>
        <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", flexWrap: "wrap", gap: "16px" }}>
          <div>
            <p style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>
              ChronoSentiment
            </p>
            <h1 style={{ fontSize: "28px", fontWeight: "700", color: "var(--text-primary)", letterSpacing: "-0.02em", margin: 0 }}>
              Decision Observatory
            </h1>
            <p style={{ fontSize: "14px", color: "var(--text-secondary)", marginTop: "8px", maxWidth: "480px" }}>
              Decisions are sealed when made. Evidence arrives later.
            </p>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            <div
              style={{
                padding: "12px 16px",
                background: "var(--bg-card)",
                border: "1px solid var(--border)",
                borderRadius: "8px",
                fontSize: "12px",
                color: "var(--text-secondary)",
                lineHeight: "1.6",
              }}
            >
              <div style={{ fontWeight: "600", color: "var(--text-primary)", marginBottom: "4px" }}>Policy</div>
              <div>C3-002 · <span style={{ fontFamily: "monospace", fontSize: "11px" }}>{shortHash("5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121")}</span></div>
              <div style={{ marginTop: "4px" }}>Status: <span style={{ color: "#8b5cf6", fontWeight: "600" }}>FROZEN</span></div>
            </div>
            <div
              style={{
                padding: "12px 16px",
                background: "var(--bg-card)",
                border: "1px solid rgba(16,185,129,0.25)",
                borderRadius: "8px",
                fontSize: "12px",
                color: "var(--text-secondary)",
                lineHeight: "1.6",
              }}
            >
              <div style={{ fontWeight: "600", color: "var(--text-primary)", marginBottom: "4px" }}>Execution Model</div>
              <div>coralys-exec-v0 · <span style={{ fontFamily: "monospace", fontSize: "11px" }}>{shortHash("3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f")}</span></div>
              <div style={{ marginTop: "4px" }}>P.E.3: <span style={{ color: "#10b981", fontWeight: "600" }}>LIVE · FROZEN</span></div>
            </div>
          </div>
        </div>
      </div>

      {/* Stats row */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))", gap: "12px", marginBottom: "40px" }}>
        {[
          { label: "Open Decisions", value: openDecisions, sub: "14 Aug 2026 cohort", color: "#3b82f6" },
          { label: "Historical Decisions", value: completedObs, sub: "Completed evidence", color: "#10b981" },
          { label: "Execution Records", value: exec.n_decisions, sub: "P.E.1 control", color: "var(--text-primary)" },
          { label: "TARGET Exits", value: targetExits, sub: "P.E.1 control", color: "#10b981" },
          { label: "HORIZON Exits", value: horizonExits, sub: "P.E.1 control", color: "#f59e0b" },
          { label: "P.E.2 Lifecycle", value: pe2.lifecycle_validation, sub: "Validation", color: "#10b981" },
        ].map((stat) => (
          <div
            key={stat.label}
            style={{
              background: "var(--bg-card)",
              border: "1px solid var(--border)",
              borderRadius: "8px",
              padding: "16px",
            }}
          >
            <div style={{ fontSize: "11px", color: "var(--text-muted)", fontWeight: "600", letterSpacing: "0.05em", textTransform: "uppercase", marginBottom: "8px" }}>
              {stat.label}
            </div>
            <div style={{ fontSize: "24px", fontWeight: "700", color: stat.color, letterSpacing: "-0.02em" }}>
              {stat.value}
            </div>
            <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>{stat.sub}</div>
          </div>
        ))}
      </div>

      {/* Two-column layout */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "24px", marginBottom: "32px" }}>
        {/* Live / Observing decisions */}
        <div>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
            <h2 style={{ fontSize: "14px", fontWeight: "600", color: "var(--text-primary)", margin: 0 }}>
              Live Cohort — 14 August 2026
            </h2>
            <span className="badge badge-observing">Observing</span>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            {liveDecisions.map((d) => (
              <Link
                key={d.decision_id}
                href={`/decisions/${d.decision_id}`}
                style={{ textDecoration: "none" }}
              >
                <div
                  className="card-hover"
                  style={{
                    background: "var(--bg-card)",
                    border: "1px solid var(--border)",
                    borderRadius: "8px",
                    padding: "12px 16px",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    cursor: "pointer",
                  }}
                >
                  <div>
                    <div style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)" }}>
                      {d.instrument}
                    </div>
                    <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                      {d.state.trend} / {d.state.momentum}
                    </div>
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <span className={d.action === "LONG" ? "badge badge-long" : d.action === "SHORT" ? "badge badge-short" : "badge badge-no-trade"}>
                      {d.action}
                    </span>
                    <div
                      style={{
                        width: "6px",
                        height: "6px",
                        borderRadius: "50%",
                        background: "#3b82f6",
                      }}
                      className="animate-pulse-slow"
                    />
                  </div>
                </div>
              </Link>
            ))}
          </div>
          <div style={{ marginTop: "12px", textAlign: "center" }}>
            <Link href="/decisions" style={{ fontSize: "12px", color: "var(--text-secondary)", textDecoration: "none" }}>
              View all decisions →
            </Link>
          </div>
        </div>

        {/* Recent completed evidence */}
        <div>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
            <h2 style={{ fontSize: "14px", fontWeight: "600", color: "var(--text-primary)", margin: 0 }}>
              Recent Evidence
            </h2>
            <span className="badge" style={{ background: "rgba(16,185,129,0.1)", color: "#10b981", border: "1px solid rgba(16,185,129,0.2)", fontSize: "11px", fontWeight: "600", padding: "2px 8px", borderRadius: "4px" }}>
              Completed
            </span>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            {recentCompleted.map((d) => {
              const obs = hist.observations.find((o) => o.decision_id === d.decision_id);
              if (!obs) return null;
              const ret = d.action === "LONG" ? obs.value_long : obs.value_short;
              const positive = ret >= 0;
              // Cross-reference for exit price
              const tick = exec.ticks.find((t) => t.decision_id === d.decision_id);
              const pe2rec = pe2.records?.find((r: { decision: { decision_id: string }; exit: { exit_price: number; exit_reason: string } }) => r.decision.decision_id === d.decision_id);
              const exitPrice = tick?.exit_price ?? pe2rec?.exit?.exit_price ?? null;
              const exitReason = tick?.exit_reason ?? pe2rec?.exit?.exit_reason ?? null;
              return (
                <Link
                  key={d.decision_id}
                  href={`/decisions/${d.decision_id}`}
                  style={{ textDecoration: "none" }}
                >
                  <div
                    className="card-hover"
                    style={{
                      background: "var(--bg-card)",
                      border: "1px solid var(--border)",
                      borderRadius: "8px",
                      padding: "12px 16px",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      cursor: "pointer",
                    }}
                  >
                    <div>
                      <div style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)" }}>
                        {d.instrument}
                      </div>
                      <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                        {formatDate(d.decision_time)}
                      </div>
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                      <span className={d.action === "LONG" ? "badge badge-long" : "badge badge-short"}>
                        {d.action}
                      </span>
                      {exitPrice !== null && (
                        <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                          Exit {formatPrice(exitPrice)}
                        </span>
                      )}
                      {exitReason && (
                        <span style={{ fontSize: "10px", fontWeight: "700", color: exitReason === "TARGET" ? "#10b981" : "#f59e0b", background: exitReason === "TARGET" ? "rgba(16,185,129,0.1)" : "rgba(245,158,11,0.1)", padding: "1px 5px", borderRadius: "3px" }}>
                          {exitReason}
                        </span>
                      )}
                      <span style={{ fontSize: "13px", fontWeight: "600", color: positive ? "#10b981" : "#ef4444" }}>
                        {formatReturn(ret)}
                      </span>
                    </div>
                  </div>
                </Link>
              );
            })}
          </div>
          <div style={{ marginTop: "12px", textAlign: "center" }}>
            <Link href="/decisions?filter=completed" style={{ fontSize: "12px", color: "var(--text-secondary)", textDecoration: "none" }}>
              View all evidence →
            </Link>
          </div>
        </div>
      </div>

      {/* Product loop diagram */}
      <div
        style={{
          background: "var(--bg-card)",
          border: "1px solid var(--border)",
          borderRadius: "12px",
          padding: "32px",
          marginBottom: "32px",
        }}
      >
        <h2 style={{ fontSize: "14px", fontWeight: "600", color: "var(--text-primary)", margin: "0 0 24px 0" }}>
          The ChronoSentiment Loop
        </h2>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: "0", flexWrap: "wrap" }}>
          {[
            { label: "Certified State", sub: "at T", color: "#3b82f6" },
            { label: "Coralys", sub: "Intelligence", color: "#8b5cf6" },
            { label: "Decision", sub: "LONG / SHORT / NO_TRADE", color: "#10b981" },
            { label: "Sealed", sub: "Immutable", color: "#8b5cf6" },
            { label: "Execution Intent", sub: "Sealed at T", color: "#3b82f6" },
            { label: "Observatory", sub: "Evidence", color: "#f59e0b" },
          ].map((step, i, arr) => (
            <div key={step.label} style={{ display: "flex", alignItems: "center" }}>
              <div style={{ textAlign: "center", padding: "0 8px" }}>
                <div
                  style={{
                    width: "10px",
                    height: "10px",
                    borderRadius: "50%",
                    background: step.color,
                    margin: "0 auto 6px",
                  }}
                />
                <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-primary)" }}>{step.label}</div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginTop: "2px" }}>{step.sub}</div>
              </div>
              {i < arr.length - 1 && (
                <div style={{ color: "var(--text-muted)", fontSize: "16px", padding: "0 4px", marginBottom: "16px" }}>→</div>
              )}
            </div>
          ))}
        </div>
        <div
          style={{
            marginTop: "24px",
            padding: "12px 16px",
            background: "rgba(59, 130, 246, 0.06)",
            border: "1px solid rgba(59, 130, 246, 0.15)",
            borderRadius: "6px",
            fontSize: "12px",
            color: "var(--text-secondary)",
            textAlign: "center",
          }}
        >
          <strong style={{ color: "var(--text-primary)" }}>Temporal guarantee:</strong>{" "}
          Same certified state at T + same frozen policy = same sealed decision. Future information cannot modify a sealed decision.
        </div>
      </div>

      {/* Integrity summary */}
      <div
        style={{
          background: "var(--bg-card)",
          border: "1px solid var(--border)",
          borderRadius: "12px",
          padding: "24px 32px",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: "16px" }}>
          <div>
            <h2 style={{ fontSize: "14px", fontWeight: "600", color: "var(--text-primary)", margin: "0 0 4px 0" }}>
              Integrity Status
            </h2>
            <p style={{ fontSize: "12px", color: "var(--text-muted)", margin: 0 }}>
              P.E.2 historical lifecycle validation
            </p>
          </div>
          <div style={{ display: "flex", gap: "12px", flexWrap: "wrap" }}>
            {[
              { label: "Determinism", pass: pe2.determinism_pass },
              { label: "No-lookahead", pass: pe2.lookahead_clean },
              { label: "Poison test", pass: pe2.poison_test_pass },
              { label: "Immutable seal", pass: !pe2.prospective_cohort_mutated },
              { label: "Append-only", pass: !pe2.protected_artifacts_mutated },
            ].map((check) => (
              <div key={check.label} style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                <div
                  style={{
                    width: "6px",
                    height: "6px",
                    borderRadius: "50%",
                    background: check.pass ? "#10b981" : "#ef4444",
                  }}
                />
                <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>{check.label}</span>
                <span style={{ fontSize: "11px", fontWeight: "600", color: check.pass ? "#10b981" : "#ef4444" }}>
                  {check.pass ? "PASS" : "FAIL"}
                </span>
              </div>
            ))}
          </div>
          <Link
            href="/audit"
            style={{
              padding: "8px 16px",
              background: "var(--bg-secondary)",
              border: "1px solid var(--border)",
              borderRadius: "6px",
              fontSize: "12px",
              color: "var(--text-secondary)",
              textDecoration: "none",
              fontWeight: "500",
            }}
          >
            View full audit →
          </Link>
        </div>
      </div>
    </div>
  );
}
