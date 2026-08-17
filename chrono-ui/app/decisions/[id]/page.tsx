import Link from "next/link";
import { promises as fs } from "fs";
import path from "path";
import { notFound } from "next/navigation";
import type { HistoricalLedger, ExecutionIntent, ExecutionReport, PE2Ledger } from "@/lib/data";
import { formatDate, formatDateTime, shortHash, formatReturn, formatPrice } from "@/lib/data";

async function getData() {
  const dataDir = path.join(process.cwd(), "public", "data");
  const [histRaw, prospRaw, intentsRaw, execRaw, pe2Raw] = await Promise.all([
    fs.readFile(path.join(dataDir, "historical_ledger.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "prospective_ledger.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "execution_intents.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "execution_report.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "pe2_replay_ledger.json"), "utf-8"),
  ]);
  return {
    hist: JSON.parse(histRaw) as HistoricalLedger,
    prosp: JSON.parse(prospRaw) as HistoricalLedger,
    intents: JSON.parse(intentsRaw) as ExecutionIntent[],
    exec: JSON.parse(execRaw) as ExecutionReport,
    pe2: JSON.parse(pe2Raw) as PE2Ledger,
  };
}

interface CertifiedDecision {
  instrument: string;
  c3_002_direction: string;
  entry_price: number;
  target_pct: number;
  target_price: number;
  risk_pct: number;
  risk_boundary: number;
  maximum_hold_sessions: number;
  decision_id: string;
  execution_intent_id: string;
}

async function getCertifiedDecisions(): Promise<CertifiedDecision[]> {
  try {
    // Server component: call backend directly (not via Next.js proxy).
    const backendUrl = process.env.CHRONOSENTIMENT_API_URL ?? "http://localhost:3000";
    const res = await fetch(`${backendUrl}/api/v0/decisions/current`, {
      cache: "no-store",
    });
    if (!res.ok) return [];
    const data = await res.json();
    return (data.decisions ?? []) as CertifiedDecision[];
  } catch {
    return [];
  }
}

export default async function DecisionDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const [{ hist, prosp, intents, exec, pe2 }, certifiedDecisions] = await Promise.all([
    getData(),
    getCertifiedDecisions(),
  ]);

  const decision =
    prosp.decisions.find((d) => d.decision_id === id) ||
    hist.decisions.find((d) => d.decision_id === id) ||
    pe2.records.find((r) => r.decision.decision_id === id)?.decision;

  if (!decision) notFound();

  // Find certified execution parameters for this instrument (from live backend).
  const certifiedForInstrument = certifiedDecisions.find(
    (cd) => cd.instrument === decision.instrument
  );

  const isProspective = prosp.decisions.some((d) => d.decision_id === id);
  const observation = hist.observations.find((o) => o.decision_id === id);
  const intent = intents.find((i) => i.decision_id === id);
  const execTick = exec.ticks.find((t) => t.decision_id === id);
  const pe2Record = pe2.records.find((r) => r.decision.decision_id === id);

  const hasEvidence = !!(observation || execTick || pe2Record);
  const ret = observation
    ? decision.action === "LONG" ? observation.value_long : observation.value_short
    : execTick
    ? execTick.decision_value
    : pe2Record
    ? pe2Record.exit.decision_value
    : null;

  const exitReason = execTick?.exit_reason ?? pe2Record?.exit.exit_reason ?? null;

  const timelineSteps = [
    {
      id: "state",
      label: "Certified State",
      sub: formatDate(decision.decision_time),
      done: true,
      color: "#3b82f6",
      detail: `${decision.state.trend} / ${decision.state.momentum} · Volatility ${decision.state.volatility}`,
    },
    {
      id: "decision",
      label: "Decision",
      sub: decision.action,
      done: true,
      color: decision.action === "LONG" ? "#10b981" : decision.action === "SHORT" ? "#ef4444" : "#6b7280",
      detail: `Policy ${decision.policy_id} applied to certified state`,
    },
    {
      id: "sealed",
      label: "Sealed",
      sub: "Immutable",
      done: true,
      color: "#8b5cf6",
      detail: `Decision ID: ${shortHash(decision.decision_id)}`,
    },
    {
      id: "intent",
      label: "Execution Intent",
      sub: intent || pe2Record
        ? "+5.0% target · 20 sessions"
        : certifiedForInstrument
        ? `Entry ${formatPrice(certifiedForInstrument.entry_price)} · Target ${formatPrice(certifiedForInstrument.target_price)}`
        : isProspective
        ? "Not attached"
        : "—",
      done: !!(intent || pe2Record || certifiedForInstrument),
      color: "#3b82f6",
      detail: intent
        ? `Entry ${formatPrice(intent.entry_price)} → Target ${formatPrice(intent.target_price)}`
        : pe2Record
        ? `Entry ${formatPrice(pe2Record.intent.entry_price)} → Target ${formatPrice(pe2Record.intent.target_price)}`
        : certifiedForInstrument
        ? `Entry ${formatPrice(certifiedForInstrument.entry_price)} → Target ${formatPrice(certifiedForInstrument.target_price)} · Stop ${formatPrice(certifiedForInstrument.risk_boundary)}`
        : isProspective
        ? "This cohort is decision-only. No execution intent attached."
        : "No execution intent for this decision.",
    },
    {
      id: "observing",
      label: "Observing",
      sub: isProspective ? "Active" : "Completed",
      done: true,
      color: isProspective ? "#3b82f6" : "#10b981",
      detail: isProspective ? "Awaiting future market sessions" : "Evidence collected",
    },
    {
      id: "evidence",
      label: "Evidence",
      sub: hasEvidence ? (exitReason ?? "Completed") : isProspective ? "Pending" : "—",
      done: hasEvidence,
      color: hasEvidence ? "#10b981" : "#4a5568",
      detail: hasEvidence && ret !== null ? `Decision value: ${formatReturn(ret)}` : "No evidence yet",
    },
  ];

  return (
    <div style={{ maxWidth: "900px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Breadcrumb */}
      <div style={{ marginBottom: "24px", display: "flex", alignItems: "center", gap: "8px", fontSize: "12px", color: "var(--text-muted)" }}>
        <Link href="/" style={{ color: "var(--text-muted)", textDecoration: "none" }}>Observatory</Link>
        <span>›</span>
        <Link href="/decisions" style={{ color: "var(--text-muted)", textDecoration: "none" }}>Decisions</Link>
        <span>›</span>
        <span style={{ color: "var(--text-secondary)" }}>{decision.instrument}</span>
      </div>

      {/* Header card */}
      <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "12px", padding: "24px", marginBottom: "24px" }}>
        <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", flexWrap: "wrap", gap: "16px" }}>
          <div>
            <div style={{ fontSize: "28px", fontWeight: "800", color: "var(--text-primary)", letterSpacing: "-0.02em" }}>
              {decision.instrument}
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: "8px", marginTop: "8px" }}>
              <span className={decision.action === "LONG" ? "badge badge-long" : decision.action === "SHORT" ? "badge badge-short" : "badge badge-no-trade"} style={{ fontSize: "13px", padding: "4px 12px" }}>
                {decision.action}
              </span>
              {isProspective ? (
                <span className="badge badge-observing">Observing</span>
              ) : (
                <span style={{ padding: "3px 8px", background: "rgba(16,185,129,0.1)", border: "1px solid rgba(16,185,129,0.2)", borderRadius: "4px", fontSize: "11px", fontWeight: "600", color: "#10b981" }}>
                  Evidence Completed
                </span>
              )}
            </div>
          </div>
          {ret !== null && (
            <div style={{ textAlign: "right" }}>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "4px" }}>Decision Value</div>
              <div style={{ fontSize: "28px", fontWeight: "800", color: ret >= 0 ? "#10b981" : "#ef4444", letterSpacing: "-0.02em" }}>
                {formatReturn(ret)}
              </div>
              {exitReason && (
                <div style={{ marginTop: "4px" }}>
                  <span className={exitReason === "TARGET" ? "badge badge-target" : "badge badge-horizon"}>{exitReason}</span>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Temporal boundary */}
      <div style={{ background: "var(--bg-card)", border: "1px solid rgba(59,130,246,0.2)", borderRadius: "12px", padding: "24px", marginBottom: "24px", position: "relative", overflow: "hidden" }}>
        <div style={{ position: "absolute", top: 0, left: 0, right: 0, height: "2px", background: "linear-gradient(90deg, #3b82f6, #8b5cf6)" }} />
        <h3 style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 20px 0" }}>
          Temporal Boundary
        </h3>
        <div style={{ display: "flex", gap: "0" }}>
          <div style={{ flex: 1, padding: "16px", background: "rgba(59,130,246,0.06)", border: "1px solid rgba(59,130,246,0.15)", borderRadius: "8px 0 0 8px" }}>
            <div style={{ fontSize: "11px", fontWeight: "600", color: "#3b82f6", marginBottom: "8px", letterSpacing: "0.05em" }}>BEFORE T — KNOWN</div>
            <div style={{ fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.7" }}>
              <div>Certified session: <strong style={{ color: "var(--text-primary)" }}>{formatDate(decision.decision_time)}</strong></div>
              <div>Market state: <strong style={{ color: "var(--text-primary)" }}>{decision.state.trend} / {decision.state.momentum}</strong></div>
              <div>Volatility: <strong style={{ color: "var(--text-primary)" }}>{decision.state.volatility}</strong></div>
              <div>Policy: <strong style={{ color: "var(--text-primary)" }}>{decision.policy_id}</strong></div>
            </div>
          </div>
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", padding: "0 16px", position: "relative" }}>
            <div style={{ width: "2px", height: "100%", background: "#8b5cf6", position: "absolute" }} />
            <div style={{ background: "#8b5cf6", color: "white", fontSize: "12px", fontWeight: "700", padding: "4px 10px", borderRadius: "4px", position: "relative", zIndex: 1, letterSpacing: "0.05em" }}>T</div>
            <div style={{ fontSize: "9px", color: "#8b5cf6", marginTop: "4px", position: "relative", zIndex: 1, fontWeight: "600", letterSpacing: "0.05em", textTransform: "uppercase" }}>SEALED</div>
          </div>
          <div style={{ flex: 1, padding: "16px", background: "rgba(107,114,128,0.04)", border: "1px solid rgba(107,114,128,0.12)", borderRadius: "0 8px 8px 0" }}>
            <div style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", marginBottom: "8px", letterSpacing: "0.05em" }}>AFTER T — FUTURE AT DECISION TIME</div>
            {hasEvidence ? (
              <div style={{ fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.7" }}>
                {execTick && (
                  <>
                    <div>Exit: <strong style={{ color: exitReason === "TARGET" ? "#10b981" : "#f59e0b" }}>{exitReason}</strong></div>
                    <div>Sessions held: <strong style={{ color: "var(--text-primary)" }}>{execTick.holding_sessions}</strong></div>
                    <div>Exit price: <strong style={{ color: "var(--text-primary)" }}>{formatPrice(execTick.exit_price)}</strong></div>
                  </>
                )}
                {pe2Record && !execTick && (
                  <>
                    <div>Exit: <strong style={{ color: pe2Record.exit.exit_reason === "TARGET" ? "#10b981" : "#f59e0b" }}>{pe2Record.exit.exit_reason}</strong></div>
                    <div>Sessions held: <strong style={{ color: "var(--text-primary)" }}>{pe2Record.exit.holding_sessions}</strong></div>
                    <div>Trigger: <strong style={{ color: "var(--text-primary)" }}>{pe2Record.exit.trigger_type}</strong></div>
                  </>
                )}
                {observation && !execTick && !pe2Record && (
                  <>
                    <div>Realized return: <strong style={{ color: ret !== null && ret >= 0 ? "#10b981" : "#ef4444" }}>{ret !== null ? formatReturn(ret) : "—"}</strong></div>
                    <div>Status: <strong style={{ color: "#10b981" }}>Completed</strong></div>
                  </>
                )}
              </div>
            ) : (
              <div style={{ fontSize: "12px", color: "var(--text-muted)", fontStyle: "italic" }}>
                Evidence not yet available.<br />The decision was made without knowledge of what follows.
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Lifecycle timeline */}
      <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "12px", padding: "24px", marginBottom: "24px" }}>
        <h3 style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 24px 0" }}>
          Lifecycle Timeline
        </h3>
        <div style={{ position: "relative" }}>
          {timelineSteps.map((step, i) => (
            <div key={step.id} style={{ display: "flex", gap: "16px" }}>
              <div style={{ display: "flex", flexDirection: "column", alignItems: "center", width: "24px", flexShrink: 0 }}>
                <div style={{ width: "12px", height: "12px", borderRadius: "50%", background: step.done ? step.color : "var(--bg-secondary)", border: `2px solid ${step.done ? step.color : "var(--border)"}`, flexShrink: 0, marginTop: "4px" }} />
                {i < timelineSteps.length - 1 && (
                  <div style={{ width: "1px", flex: 1, minHeight: "32px", background: step.done ? "var(--border)" : "var(--border-subtle)", margin: "4px 0" }} />
                )}
              </div>
              <div style={{ flex: 1, paddingBottom: i < timelineSteps.length - 1 ? "20px" : "0" }}>
                <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                  <span style={{ fontSize: "13px", fontWeight: "600", color: step.done ? "var(--text-primary)" : "var(--text-muted)" }}>{step.label}</span>
                  <span style={{ fontSize: "11px", fontWeight: "600", color: step.done ? step.color : "var(--text-muted)", background: step.done ? `${step.color}18` : "transparent", padding: "1px 6px", borderRadius: "3px" }}>{step.sub}</span>
                </div>
                <div style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "4px" }}>{step.detail}</div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Two-column detail */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px", marginBottom: "16px" }}>
        {/* Decision snapshot */}
        <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "12px", padding: "20px" }}>
          <h3 style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 16px 0" }}>Decision Snapshot</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
            {[
              { label: "Decision Time", value: formatDateTime(decision.decision_time) },
              { label: "Policy", value: decision.policy_id },
              { label: "Engine", value: decision.engine_version },
              { label: "Horizon", value: `${decision.horizon_days} market sessions` },
              { label: "Paper Only", value: decision.paper_only ? "Yes" : "No" },
            ].map((row) => (
              <div key={row.label} style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                <span style={{ fontSize: "12px", color: "var(--text-muted)" }}>{row.label}</span>
                <span style={{ fontSize: "12px", color: "var(--text-primary)", fontWeight: "500", textAlign: "right", maxWidth: "200px" }}>{row.value}</span>
              </div>
            ))}
            <div style={{ paddingTop: "8px", borderTop: "1px solid var(--border-subtle)" }}>
              <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>Decision ID</div>
              <div style={{ fontSize: "10px", fontFamily: "monospace", color: "var(--text-secondary)", wordBreak: "break-all" }}>{decision.decision_id}</div>
            </div>
            <div>
              <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>Policy Artifact</div>
              <div style={{ fontSize: "10px", fontFamily: "monospace", color: "var(--text-secondary)", wordBreak: "break-all" }}>{decision.policy_artifact_sha256}</div>
            </div>
          </div>
        </div>

        {/* Certified State */}
        <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "12px", padding: "20px" }}>
          <h3 style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 16px 0" }}>Certified State at T</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
            <div style={{ padding: "12px", background: "rgba(59,130,246,0.06)", border: "1px solid rgba(59,130,246,0.12)", borderRadius: "6px" }}>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "4px" }}>Market State</div>
              <div style={{ fontSize: "16px", fontWeight: "700", color: "var(--text-primary)" }}>{decision.state.trend} / {decision.state.momentum}</div>
            </div>
            {[
              { label: "Trend", value: decision.state.trend },
              { label: "Momentum", value: decision.state.momentum },
              { label: "Volatility", value: decision.state.volatility },
              { label: "Input Schema", value: decision.state.input_schema.join(", ") },
            ].map((row) => (
              <div key={row.label} style={{ display: "flex", justifyContent: "space-between" }}>
                <span style={{ fontSize: "12px", color: "var(--text-muted)" }}>{row.label}</span>
                <span style={{ fontSize: "12px", color: "var(--text-primary)", fontWeight: "500" }}>{row.value}</span>
              </div>
            ))}
            <div style={{ paddingTop: "8px", borderTop: "1px solid var(--border-subtle)" }}>
              <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>State Hash</div>
              <div style={{ fontSize: "10px", fontFamily: "monospace", color: "var(--text-secondary)", wordBreak: "break-all" }}>{decision.state.state_hash}</div>
            </div>
          </div>
        </div>
      </div>

      {/* Price-anchored execution record — three seals */}
      {(intent || pe2Record || execTick) && (
        <div style={{ background: "var(--bg-card)", border: "1px solid rgba(59,130,246,0.15)", borderRadius: "12px", padding: "24px", marginBottom: "16px" }}>
          <h3 style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 20px 0" }}>
            Execution Record
          </h3>

          {/* Three-seal flow */}
          <div style={{ display: "grid", gridTemplateColumns: "1fr auto 1fr auto 1fr", gap: "0", alignItems: "start", marginBottom: "20px" }}>

            {/* Seal 1 — Decision */}
            <div style={{ background: "rgba(139,92,246,0.06)", border: "1px solid rgba(139,92,246,0.15)", borderRadius: "8px", padding: "14px" }}>
              <div style={{ fontSize: "9px", fontWeight: "700", color: "#8b5cf6", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>① Decision Seal</div>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Direction</div>
              <div style={{ fontSize: "16px", fontWeight: "800", color: decision.action === "LONG" ? "#10b981" : decision.action === "SHORT" ? "#ef4444" : "#6b7280", marginBottom: "8px" }}>{decision.action}</div>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Certified at</div>
              <div style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-secondary)", marginBottom: "8px" }}>{formatDate(decision.decision_time)}</div>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Policy</div>
              <div style={{ fontSize: "11px", fontWeight: "600", color: "#8b5cf6" }}>{decision.policy_id}</div>
            </div>

            {/* Arrow */}
            <div style={{ display: "flex", alignItems: "center", justifyContent: "center", padding: "0 8px", paddingTop: "32px" }}>
              <span style={{ fontSize: "16px", color: "var(--text-muted)" }}>→</span>
            </div>

            {/* Seal 2 — Execution */}
            <div style={{ background: "rgba(59,130,246,0.06)", border: "1px solid rgba(59,130,246,0.15)", borderRadius: "8px", padding: "14px" }}>
              <div style={{ fontSize: "9px", fontWeight: "700", color: "#3b82f6", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>② Execution Seal</div>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Entry price</div>
              <div style={{ fontSize: "16px", fontWeight: "800", color: "var(--text-primary)", marginBottom: "8px" }}>
                {intent ? formatPrice(intent.entry_price) : pe2Record ? formatPrice(pe2Record.intent.entry_price) : execTick ? formatPrice(execTick.entry_price) : certifiedForInstrument ? formatPrice(certifiedForInstrument.entry_price) : "—"}
              </div>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Target price</div>
              <div style={{ fontSize: "11px", fontWeight: "600", color: "#10b981", marginBottom: "8px" }}>
                {intent ? formatPrice(intent.target_price) : pe2Record ? formatPrice(pe2Record.intent.target_price) : execTick ? formatPrice(execTick.target_price) : certifiedForInstrument ? formatPrice(certifiedForInstrument.target_price) : "—"}
                {" "}
                <span style={{ color: "var(--text-muted)", fontWeight: "400" }}>
                  ({intent ? `+${(intent.target_pct * 100).toFixed(2)}%` : pe2Record ? `+${(pe2Record.intent.target_pct * 100).toFixed(2)}%` : execTick ? `+${(execTick.target_pct * 100).toFixed(2)}%` : certifiedForInstrument ? `+${(certifiedForInstrument.target_pct * 100).toFixed(2)}%` : ""})
                </span>
              </div>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Stop (risk boundary)</div>
              <div style={{ fontSize: "11px", fontWeight: "600", color: "#ef4444" }}>
                {intent?.stop_price != null
                  ? formatPrice(intent.stop_price)
                  : pe2Record?.intent?.stop_price != null
                  ? formatPrice(pe2Record.intent.stop_price)
                  : certifiedForInstrument
                  ? `${formatPrice(certifiedForInstrument.risk_boundary)} (−${(certifiedForInstrument.risk_pct * 100).toFixed(2)}%)`
                  : "Not authorized"}
              </div>
              <div style={{ marginTop: "8px", paddingTop: "8px", borderTop: "1px solid var(--border-subtle)" }}>
                <div style={{ fontSize: "9px", color: "var(--text-muted)", marginBottom: "2px" }}>Contract</div>
                <div style={{ fontSize: "10px", fontFamily: "monospace", color: "var(--text-muted)" }}>
                  {intent?.execution_contract ?? pe2Record?.intent?.execution_contract ?? (certifiedForInstrument ? "coralys-exec-v0 · ATR/TMV · 20 sessions" : "Execution Contract v0")}
                </div>
              </div>
            </div>

            {/* Arrow */}
            <div style={{ display: "flex", alignItems: "center", justifyContent: "center", padding: "0 8px", paddingTop: "32px" }}>
              <span style={{ fontSize: "16px", color: "var(--text-muted)" }}>→</span>
            </div>

            {/* Seal 3 — Exit / Evidence */}
            <div style={{
              background: hasEvidence ? "rgba(16,185,129,0.06)" : "rgba(107,114,128,0.04)",
              border: `1px solid ${hasEvidence ? "rgba(16,185,129,0.15)" : "rgba(107,114,128,0.12)"}`,
              borderRadius: "8px",
              padding: "14px",
            }}>
              <div style={{ fontSize: "9px", fontWeight: "700", color: hasEvidence ? "#10b981" : "#6b7280", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>③ Exit / Evidence</div>
              {execTick ? (
                <>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Exit reason</div>
                  <div style={{ fontSize: "16px", fontWeight: "800", color: execTick.exit_reason === "TARGET" ? "#10b981" : "#f59e0b", marginBottom: "8px" }}>{execTick.exit_reason}</div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Exit price</div>
                  <div style={{ fontSize: "14px", fontWeight: "700", color: "var(--text-primary)", marginBottom: "8px" }}>{formatPrice(execTick.exit_price)}</div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Holding period</div>
                  <div style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-secondary)", marginBottom: "8px" }}>{execTick.holding_sessions} sessions</div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Decision value</div>
                  <div style={{ fontSize: "16px", fontWeight: "800", color: execTick.decision_value >= 0 ? "#10b981" : "#ef4444" }}>{formatReturn(execTick.decision_value)}</div>
                </>
              ) : pe2Record ? (
                <>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Exit reason</div>
                  <div style={{ fontSize: "16px", fontWeight: "800", color: pe2Record.exit.exit_reason === "TARGET" ? "#10b981" : "#f59e0b", marginBottom: "8px" }}>{pe2Record.exit.exit_reason}</div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Exit price</div>
                  <div style={{ fontSize: "14px", fontWeight: "700", color: "var(--text-primary)", marginBottom: "8px" }}>{formatPrice(pe2Record.exit.exit_price)}</div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Trigger</div>
                  <div style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-secondary)", marginBottom: "8px" }}>{pe2Record.exit.trigger_type}</div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Holding period</div>
                  <div style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-secondary)", marginBottom: "8px" }}>{pe2Record.exit.holding_sessions} sessions</div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Decision value</div>
                  <div style={{ fontSize: "16px", fontWeight: "800", color: pe2Record.exit.decision_value >= 0 ? "#10b981" : "#ef4444" }}>{formatReturn(pe2Record.exit.decision_value)}</div>
                </>
              ) : observation && !execTick && !pe2Record ? (
                <>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Status</div>
                  <div style={{ fontSize: "16px", fontWeight: "800", color: "#10b981", marginBottom: "8px" }}>Completed</div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>Decision value</div>
                  <div style={{ fontSize: "16px", fontWeight: "800", color: ret !== null && ret >= 0 ? "#10b981" : "#ef4444" }}>{ret !== null ? formatReturn(ret) : "—"}</div>
                </>
              ) : (
                <div style={{ fontSize: "12px", color: "var(--text-muted)", fontStyle: "italic", lineHeight: "1.5" }}>
                  Awaiting future market sessions.<br />Evidence will be recorded here.
                </div>
              )}
            </div>
          </div>

          {/* Intent hash */}
          {(intent?.intent_hash ?? pe2Record?.intent?.intent_hash) && (
            <div style={{ paddingTop: "12px", borderTop: "1px solid var(--border-subtle)" }}>
              <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>Execution Intent Hash</div>
              <div style={{ fontSize: "10px", fontFamily: "monospace", color: "var(--text-secondary)" }}>{intent?.intent_hash ?? pe2Record?.intent.intent_hash}</div>
            </div>
          )}
        </div>
      )}

      {/* Historical observation only (no execution intent) */}
      {observation && !intent && !pe2Record && !execTick && (
        <div style={{ background: "var(--bg-card)", border: "1px solid rgba(16,185,129,0.2)", borderRadius: "12px", padding: "20px", marginBottom: "16px" }}>
          <h3 style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 16px 0" }}>Evidence</h3>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))", gap: "16px", marginBottom: "12px" }}>
            <div>
              <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>Status</div>
              <div style={{ fontSize: "16px", fontWeight: "700", color: "#10b981" }}>Completed</div>
            </div>
            <div>
              <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>Decision Value</div>
              <div style={{ fontSize: "16px", fontWeight: "700", color: ret !== null && ret >= 0 ? "#10b981" : "#ef4444" }}>{ret !== null ? formatReturn(ret) : "—"}</div>
            </div>
            <div>
              <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>Entry Price</div>
              <div style={{ fontSize: "13px", color: "var(--text-muted)", fontStyle: "italic" }}>Not in ledger</div>
            </div>
            <div>
              <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>Exit Price</div>
              <div style={{ fontSize: "13px", color: "var(--text-muted)", fontStyle: "italic" }}>Not in ledger</div>
            </div>
          </div>
          <div style={{ padding: "8px 12px", background: "rgba(107,114,128,0.06)", border: "1px solid rgba(107,114,128,0.1)", borderRadius: "6px", fontSize: "11px", color: "var(--text-muted)", lineHeight: "1.5" }}>
            This is a historical observation record. Entry and exit prices are not stored in the observation ledger — only the decision value (realized return) is recorded. Price-anchored execution records are available for P.E.1 and P.E.2 decisions.
          </div>
        </div>
      )}

      {/* Nav footer */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", paddingTop: "16px" }}>
        <Link href="/decisions" style={{ fontSize: "12px", color: "var(--text-secondary)", textDecoration: "none" }}>← Back to Decisions</Link>
        <Link href="/replay" style={{ fontSize: "12px", color: "var(--text-secondary)", textDecoration: "none" }}>View Historical Replay →</Link>
      </div>
    </div>
  );
}
