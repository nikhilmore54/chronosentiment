import { promises as fs } from "fs";
import path from "path";
import type { HistoricalLedger, PE2Ledger } from "@/lib/data";

async function getData() {
  const dataDir = path.join(process.cwd(), "public", "data");
  const [histRaw, pe2Raw] = await Promise.all([
    fs.readFile(path.join(dataDir, "historical_ledger.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "pe2_replay_ledger.json"), "utf-8"),
  ]);
  return {
    hist: JSON.parse(histRaw) as HistoricalLedger,
    pe2: JSON.parse(pe2Raw) as PE2Ledger,
  };
}

function HashBlock({ label, value, expandable }: { label: string; value: string; expandable?: boolean }) {
  return (
    <div style={{ marginBottom: "12px" }}>
      <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px", fontWeight: "600", letterSpacing: "0.05em", textTransform: "uppercase" }}>{label}</div>
      <div
        style={{
          fontFamily: "monospace",
          fontSize: "11px",
          color: "var(--text-secondary)",
          background: "var(--bg-secondary)",
          border: "1px solid var(--border-subtle)",
          borderRadius: "4px",
          padding: "8px 10px",
          wordBreak: "break-all",
          lineHeight: "1.5",
        }}
      >
        {value}
      </div>
      {expandable && (
        <div style={{ fontSize: "10px", color: "var(--text-muted)", marginTop: "3px" }}>Full hash shown above</div>
      )}
    </div>
  );
}

export default async function ProvenancePage() {
  const { hist, pe2 } = await getData();

  return (
    <div style={{ maxWidth: "900px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Header */}
      <div style={{ marginBottom: "32px" }}>
        <p style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>Observatory</p>
        <h1 style={{ fontSize: "24px", fontWeight: "700", color: "var(--text-primary)", letterSpacing: "-0.02em", margin: "0 0 8px 0" }}>Policy & Provenance</h1>
        <p style={{ fontSize: "13px", color: "var(--text-secondary)", margin: 0 }}>
          The frozen policy artifact, execution contract, and data lineage that underpin every decision.
        </p>
      </div>

      {/* Policy card */}
      <div
        style={{
          background: "var(--bg-card)",
          border: "1px solid rgba(139,92,246,0.25)",
          borderRadius: "12px",
          padding: "24px",
          marginBottom: "20px",
          position: "relative",
          overflow: "hidden",
        }}
      >
        <div style={{ position: "absolute", top: 0, left: 0, right: 0, height: "2px", background: "linear-gradient(90deg, #8b5cf6, #3b82f6)" }} />
        <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", flexWrap: "wrap", gap: "16px", marginBottom: "20px" }}>
          <div>
            <h2 style={{ fontSize: "20px", fontWeight: "800", color: "var(--text-primary)", margin: "0 0 4px 0", letterSpacing: "-0.01em" }}>C3-002</h2>
            <p style={{ fontSize: "12px", color: "var(--text-secondary)", margin: 0 }}>Direction policy · LONG / SHORT / NO_TRADE</p>
          </div>
          <div style={{ display: "flex", gap: "8px" }}>
            <span
              style={{
                padding: "4px 12px",
                background: "rgba(139,92,246,0.12)",
                border: "1px solid rgba(139,92,246,0.25)",
                borderRadius: "4px",
                fontSize: "11px",
                fontWeight: "700",
                color: "#8b5cf6",
                letterSpacing: "0.05em",
                textTransform: "uppercase",
              }}
            >
              FROZEN
            </span>
          </div>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px", marginBottom: "20px" }}>
          {[
            { label: "Policy ID", value: hist.policy_id },
            { label: "Contract ID", value: hist.contract_id },
            { label: "Paper Only", value: hist.paper_only ? "Yes — research / paper mode" : "No" },
            { label: "Path Kind", value: hist.path_kind },
            { label: "Search Three", value: hist.search_three_authorized ? "Authorized" : "Not authorized" },
            { label: "Regime Persistence", value: hist.regime_persistence_experiment_authorized ? "Authorized" : "Not authorized" },
          ].map((row) => (
            <div key={row.label} style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", padding: "8px 0", borderBottom: "1px solid var(--border-subtle)" }}>
              <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>{row.label}</span>
              <span style={{ fontSize: "11px", color: "var(--text-primary)", fontWeight: "500", textAlign: "right" }}>{row.value}</span>
            </div>
          ))}
        </div>

        <HashBlock label="Policy Artifact SHA-256" value={hist.policy_artifact_sha256} expandable />

        <div
          style={{
            padding: "12px 14px",
            background: "rgba(139,92,246,0.06)",
            border: "1px solid rgba(139,92,246,0.12)",
            borderRadius: "6px",
            fontSize: "12px",
            color: "var(--text-secondary)",
            lineHeight: "1.6",
          }}
        >
          <strong style={{ color: "var(--text-primary)" }}>What C3-002 does:</strong> C3-002 is a direction policy. It receives the certified state (Trend, Momentum, Volatility) and produces LONG, SHORT, or NO_TRADE. It does not contain the +5% target — that belongs to Execution Contract v0.
        </div>
      </div>

      {/* Execution Contract — P.E.2 control baseline */}
      <div style={{ background: "var(--bg-card)", border: "1px solid rgba(59,130,246,0.2)", borderRadius: "12px", padding: "24px", marginBottom: "20px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "16px" }}>
          <h2 style={{ fontSize: "16px", fontWeight: "700", color: "var(--text-primary)", margin: 0 }}>Execution Contract v0</h2>
          <span style={{ padding: "3px 8px", background: "rgba(59,130,246,0.1)", border: "1px solid rgba(59,130,246,0.2)", borderRadius: "4px", fontSize: "10px", fontWeight: "700", color: "#3b82f6", letterSpacing: "0.05em" }}>CONTROL · P.E.2</span>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px", marginBottom: "16px" }}>
          {[
            { label: "Target", value: "+5.0% (fixed)" },
            { label: "Maximum Hold", value: "20 market sessions" },
            { label: "Exit Model", value: "TARGET or HORIZON" },
            { label: "Stop Loss", value: "Not authorized" },
            { label: "Path Optimization", value: "Not authorized" },
            { label: "Calendar Basis", value: "Trading days (weekends & holidays excluded)" },
          ].map((row) => (
            <div key={row.label} style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", padding: "8px 0", borderBottom: "1px solid var(--border-subtle)" }}>
              <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>{row.label}</span>
              <span style={{ fontSize: "11px", color: "var(--text-primary)", fontWeight: "500", textAlign: "right" }}>{row.value}</span>
            </div>
          ))}
        </div>
        <HashBlock label="Contract ID" value={pe2.execution_contract} />
        <div style={{ padding: "10px 14px", background: "rgba(59,130,246,0.05)", border: "1px solid rgba(59,130,246,0.1)", borderRadius: "6px", fontSize: "11px", color: "var(--text-muted)", lineHeight: "1.6" }}>
          The +5% target is a fixed control parameter for P.E.2. It is not derived by Coralys and has not been optimized. Used as the baseline against which coralys-exec-v0 (P.E.3) is evaluated.
        </div>
      </div>

      {/* P.E.3 — coralys-exec-v0 */}
      <div
        style={{
          background: "var(--bg-card)",
          border: "1px solid rgba(16,185,129,0.25)",
          borderRadius: "12px",
          padding: "24px",
          marginBottom: "20px",
          position: "relative",
          overflow: "hidden",
        }}
      >
        <div style={{ position: "absolute", top: 0, left: 0, right: 0, height: "2px", background: "linear-gradient(90deg, #10b981, #3b82f6)" }} />
        <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "16px", flexWrap: "wrap" }}>
          <h2 style={{ fontSize: "16px", fontWeight: "700", color: "var(--text-primary)", margin: 0 }}>coralys-exec-v0</h2>
          <span style={{ padding: "3px 8px", background: "rgba(16,185,129,0.12)", border: "1px solid rgba(16,185,129,0.3)", borderRadius: "4px", fontSize: "10px", fontWeight: "700", color: "#10b981", letterSpacing: "0.05em" }}>LIVE · FROZEN</span>
          <span style={{ padding: "3px 8px", background: "rgba(59,130,246,0.08)", border: "1px solid rgba(59,130,246,0.2)", borderRadius: "4px", fontSize: "10px", fontWeight: "700", color: "#3b82f6", letterSpacing: "0.05em" }}>P.E.3</span>
        </div>
        <div style={{ fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.7", marginBottom: "16px" }}>
          A frozen ATR/TMV execution heuristic. Derives target% and risk% from ATR(14) and the certified TMV state at T. Not a learned model — multipliers are frozen design parameters sealed in the artifact hash below.
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px", marginBottom: "16px" }}>
          {[
            { label: "Model ID", value: "coralys-exec-v0" },
            { label: "Version", value: "0.1.0" },
            { label: "Entry Source", value: "NEXT_SESSION_OPEN" },
            { label: "Maximum Hold", value: "20 market sessions" },
            { label: "Target range", value: "2% – 15% of entry" },
            { label: "Risk range", value: "1% – 8% of entry" },
            { label: "Direction owner", value: "C3-002 (unchanged)" },
            { label: "Frozen", value: "2026-08-16" },
          ].map((row) => (
            <div key={row.label} style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", padding: "8px 0", borderBottom: "1px solid var(--border-subtle)" }}>
              <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>{row.label}</span>
              <span style={{ fontSize: "11px", color: "var(--text-primary)", fontWeight: "500", textAlign: "right" }}>{row.value}</span>
            </div>
          ))}
        </div>
        <div style={{ marginBottom: "12px" }}>
          <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "6px", fontWeight: "600", letterSpacing: "0.05em", textTransform: "uppercase" }}>TMV Multipliers (frozen)</div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "6px" }}>
            {[
              { state: "BullishPositive", target: "2.0×", risk: "1.0×" },
              { state: "BullishNegative", target: "1.5×", risk: "0.75×" },
              { state: "BearishPositive", target: "1.5×", risk: "0.75×" },
              { state: "BearishNegative", target: "1.0×", risk: "0.5×" },
            ].map((m) => (
              <div key={m.state} style={{ padding: "8px", background: "var(--bg-secondary)", border: "1px solid var(--border-subtle)", borderRadius: "6px", textAlign: "center" }}>
                <div style={{ fontSize: "9px", color: "var(--text-muted)", marginBottom: "4px", fontWeight: "600" }}>{m.state}</div>
                <div style={{ fontSize: "11px", color: "#10b981", fontWeight: "700" }}>T {m.target}</div>
                <div style={{ fontSize: "11px", color: "#ef4444", fontWeight: "700" }}>R {m.risk}</div>
              </div>
            ))}
          </div>
        </div>
        <HashBlock label="Execution Model Artifact SHA-256 (sections 1–11)" value="3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f" expandable />
        <div style={{ padding: "10px 14px", background: "rgba(16,185,129,0.05)", border: "1px solid rgba(16,185,129,0.12)", borderRadius: "6px", fontSize: "11px", color: "var(--text-muted)", lineHeight: "1.6" }}>
          <strong style={{ color: "var(--text-primary)" }}>Two information boundaries:</strong> Direction sealed by C3-002 at T (last certified session). Execution sealed at E (next eligible session open). coralys-exec-v0 must not access any information after T.
        </div>
      </div>

      {/* Coralys */}
      <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "12px", padding: "24px", marginBottom: "20px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "16px" }}>
          <h2 style={{ fontSize: "16px", fontWeight: "700", color: "var(--text-primary)", margin: 0 }}>Coralys</h2>
          <span style={{ padding: "3px 8px", background: "rgba(107,114,128,0.1)", border: "1px solid rgba(107,114,128,0.2)", borderRadius: "4px", fontSize: "10px", fontWeight: "700", color: "#9ca3af", letterSpacing: "0.05em" }}>INTELLIGENCE LAYER</span>
        </div>
        <div style={{ fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.7", marginBottom: "16px" }}>
          Coralys interprets the certified information state available at T. It produces the intelligence that feeds into the decision engine.
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          {[
            { label: "Current role", value: "Interprets certified state → feeds C3-002" },
            { label: "Execution target derivation", value: "coralys-exec-v0 — LIVE (P.E.3, frozen 2026-08-16)" },
            { label: "Adaptive targets", value: "ATR(14) × TMV multiplier, clamped — see coralys-exec-v0 above" },
            { label: "Direction", value: "C3-002 owns direction — Coralys does not override" },
          ].map((row) => (
            <div key={row.label} style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", padding: "8px 0", borderBottom: "1px solid var(--border-subtle)" }}>
              <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>{row.label}</span>
              <span style={{ fontSize: "11px", color: "var(--text-primary)", fontWeight: "500", textAlign: "right", maxWidth: "300px" }}>{row.value}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Frozen research boundaries */}
      <div style={{ background: "var(--bg-card)", border: "1px solid rgba(245,158,11,0.15)", borderRadius: "12px", padding: "24px", marginBottom: "20px" }}>
        <h2 style={{ fontSize: "13px", fontWeight: "600", color: "#f59e0b", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 16px 0" }}>Frozen Research Boundaries</h2>
        <p style={{ fontSize: "12px", color: "var(--text-secondary)", margin: "0 0 12px 0" }}>
          The following are frozen and must not be reopened during the demo build:
        </p>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "6px" }}>
          {[
            "C.3-F", "C3-002", "Search #2", "Search #3", "C.3-G",
            "Universe", "TMV definition", "MOGA research", "Regime detector",
            "Additional indicators", "Policy retuning", "Path-optimized targets",
            "Stop-loss research", "Real-money execution",
          ].map((item) => (
            <div key={item} style={{ display: "flex", alignItems: "center", gap: "6px", padding: "4px 0" }}>
              <div style={{ width: "4px", height: "4px", borderRadius: "50%", background: "#f59e0b", flexShrink: 0 }} />
              <span style={{ fontSize: "11px", color: "var(--text-muted)", fontFamily: "monospace" }}>{item}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Protected artifacts */}
      <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "12px", padding: "24px" }}>
        <h2 style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.08em", textTransform: "uppercase", margin: "0 0 16px 0" }}>Protected Artifacts</h2>
        <p style={{ fontSize: "12px", color: "var(--text-secondary)", margin: "0 0 12px 0" }}>
          These artifacts are immutable. They must never be rewritten to make the product look better.
        </p>
        <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          {[
            { label: "14-August prospective cohort", desc: "Decision-only, no execution intent attached" },
            { label: "P.E.1 — Targeted Execution Replay", desc: "14 intents, May–June 2026" },
            { label: "Replay v0", desc: "Archived — 20 calendar day horizon (superseded)" },
            { label: "Replay v1", desc: "20 market session horizon (current)" },
            { label: "P.E.2 historical sidecar", desc: "7 intents, July 2026, lifecycle validation PASS" },
            { label: "C3-002 artifact", desc: `SHA-256: ${hist.policy_artifact_sha256.slice(0, 16)}…` },
            { label: "P.E.3 — coralys-exec-v0 spec (sections 1–11)", desc: "SHA-256: 3876ffa232f75068… · frozen 2026-08-16" },
          ].map((item) => (
            <div key={item.label} style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", padding: "10px 12px", background: "var(--bg-secondary)", border: "1px solid var(--border-subtle)", borderRadius: "6px" }}>
              <div>
                <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-primary)" }}>{item.label}</div>
                <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>{item.desc}</div>
              </div>
              <div style={{ width: "6px", height: "6px", borderRadius: "50%", background: "#8b5cf6", flexShrink: 0, marginTop: "4px" }} />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}