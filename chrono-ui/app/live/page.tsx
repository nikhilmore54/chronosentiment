import Link from "next/link";
import { promises as fs } from "fs";
import path from "path";
import type { LiveExecutionLedger, LivePosition } from "@/lib/data";
import { formatDate, formatPrice, formatReturn } from "@/lib/data";

async function getData() {
  const dataDir = path.join(process.cwd(), "public", "data");
  const raw = await fs.readFile(path.join(dataDir, "live_execution_ledger.json"), "utf-8");
  return { ledger: JSON.parse(raw) as LiveExecutionLedger };
}

function getMarketStatus(): { status: string; label: string; color: string; next: string } {
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

function lifecycleColor(state: string): string {
  switch (state) {
    case "DECISION_ONLY": return "#6b7280";
    case "READY_TO_ENTER": return "#f59e0b";
    case "ACTIVE": return "#3b82f6";
    case "EXITED": return "#10b981";
    default: return "#6b7280";
  }
}

function lifecycleLabel(state: string): string {
  switch (state) {
    case "DECISION_ONLY": return "Decision Only";
    case "READY_TO_ENTER": return "Ready to Enter";
    case "ACTIVE": return "Active";
    case "EXITED": return "Exited";
    default: return state;
  }
}

function PositionCard({ pos }: { pos: LivePosition }) {
  const color = lifecycleColor(pos.lifecycle_state);
  const isActive = pos.lifecycle_state === "ACTIVE";
  const isExited = pos.lifecycle_state === "EXITED";
  const isReady = pos.lifecycle_state === "READY_TO_ENTER";

  return (
    <div style={{
      background: "var(--bg-card)",
      border: `1px solid ${isActive ? "rgba(59,130,246,0.25)" : isExited ? "rgba(16,185,129,0.2)" : "var(--border)"}`,
      borderRadius: "12px",
      overflow: "hidden",
    }}>
      {/* Top accent */}
      <div style={{ height: "2px", background: color }} />

      <div style={{ padding: "16px 20px" }}>
        {/* Header row */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "14px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <span style={{ fontSize: "15px", fontWeight: "700", color: "var(--text-primary)" }}>{pos.instrument}</span>
            <span className={pos.direction === "LONG" ? "badge badge-long" : pos.direction === "SHORT" ? "badge badge-short" : "badge badge-no-trade"}>
              {pos.direction}
            </span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            {isActive && <div style={{ width: "6px", height: "6px", borderRadius: "50%", background: "#3b82f6" }} className="animate-pulse-slow" />}
            <span style={{ fontSize: "11px", fontWeight: "700", color, background: `${color}18`, padding: "2px 8px", borderRadius: "4px", letterSpacing: "0.05em" }}>
              {lifecycleLabel(pos.lifecycle_state)}
            </span>
          </div>
        </div>

        {/* State-specific content */}
        {pos.lifecycle_state === "DECISION_ONLY" && (
          <div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px", marginBottom: "10px" }}>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Market State</div>
                <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
                  {pos.decision_seal.market_state.trend} / {pos.decision_seal.market_state.momentum}
                </div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Certified</div>
                <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--text-secondary)" }}>
                  {formatDate(pos.decision_seal.decision_timestamp)}
                </div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Entry Price</div>
                <div style={{ fontSize: "12px", color: "var(--text-muted)", fontStyle: "italic" }}>Awaiting fill</div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Execution Intent</div>
                <div style={{ fontSize: "12px", color: "var(--text-muted)", fontStyle: "italic" }}>Not attached</div>
              </div>
            </div>
            {pos.next_eligible_session && (
              <div style={{ padding: "8px 10px", background: "rgba(107,114,128,0.06)", border: "1px solid rgba(107,114,128,0.1)", borderRadius: "6px", fontSize: "10px", color: "var(--text-muted)" }}>
                Next eligible session: <strong style={{ color: "var(--text-secondary)" }}>{pos.next_eligible_session}</strong>
              </div>
            )}
          </div>
        )}

        {pos.lifecycle_state === "READY_TO_ENTER" && pos.execution_seal && (
          <div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "12px" }}>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Target</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "#10b981" }}>{formatPrice(pos.execution_seal.target_price)}</div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>+{(pos.execution_seal.target_pct * 100).toFixed(2)}%</div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Risk Boundary</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "#f59e0b" }}>
                  {pos.execution_seal.risk_boundary !== null ? formatPrice(pos.execution_seal.risk_boundary) : "Not authorized"}
                </div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Max Hold</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-primary)" }}>{pos.execution_seal.maximum_hold_sessions} sessions</div>
              </div>
            </div>
            <div style={{ marginTop: "10px", padding: "8px 10px", background: "rgba(245,158,11,0.06)", border: "1px solid rgba(245,158,11,0.12)", borderRadius: "6px", fontSize: "10px", color: "#f59e0b" }}>
              Execution intent sealed. Awaiting entry fill at next session open.
            </div>
          </div>
        )}

        {pos.lifecycle_state === "ACTIVE" && pos.execution_seal && pos.monitoring && (
          <div>
            {/* Price bar */}
            <div style={{ marginBottom: "14px" }}>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
                <span style={{ fontSize: "10px", color: "#ef4444" }}>
                  RISK {pos.execution_seal.risk_boundary !== null ? formatPrice(pos.execution_seal.risk_boundary) : "—"}
                </span>
                <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>
                  ENTRY {formatPrice(pos.execution_seal.entry_price)}
                </span>
                <span style={{ fontSize: "10px", color: "#10b981" }}>
                  TARGET {formatPrice(pos.execution_seal.target_price)}
                </span>
              </div>
              <div style={{ height: "4px", background: "var(--bg-secondary)", borderRadius: "2px", position: "relative" }}>
                <div style={{ position: "absolute", left: "50%", top: "-2px", width: "8px", height: "8px", borderRadius: "50%", background: "#3b82f6", transform: "translateX(-50%)" }} />
              </div>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: "10px" }}>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Entry</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-primary)" }}>{formatPrice(pos.execution_seal.entry_price)}</div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Target</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "#10b981" }}>{formatPrice(pos.execution_seal.target_price)}</div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Risk Boundary</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "#f59e0b" }}>
                  {pos.execution_seal.risk_boundary !== null ? formatPrice(pos.execution_seal.risk_boundary) : "—"}
                </div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Session</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "#3b82f6" }}>
                  {pos.monitoring.sessions_elapsed} / {pos.execution_seal.maximum_hold_sessions}
                </div>
              </div>
            </div>
          </div>
        )}

        {pos.lifecycle_state === "EXITED" && pos.execution_seal && pos.exit_record && (
          <div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: "10px" }}>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Entry</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-primary)" }}>{formatPrice(pos.execution_seal.entry_price)}</div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Exit</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: "var(--text-primary)" }}>{formatPrice(pos.exit_record.exit_price)}</div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Exit Reason</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: pos.exit_record.exit_reason === "TARGET" ? "#10b981" : pos.exit_record.exit_reason === "RISK" ? "#ef4444" : "#f59e0b" }}>
                  {pos.exit_record.exit_reason}
                </div>
              </div>
              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Decision Value</div>
                <div style={{ fontSize: "13px", fontWeight: "700", color: pos.exit_record.decision_value >= 0 ? "#10b981" : "#ef4444" }}>
                  {formatReturn(pos.exit_record.decision_value)}
                </div>
              </div>
            </div>
            {pos.exit_record.trigger_session_ohlc && (
              <div style={{ marginTop: "10px", padding: "8px 10px", background: "rgba(16,185,129,0.06)", border: "1px solid rgba(16,185,129,0.12)", borderRadius: "6px" }}>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "4px" }}>Trigger session OHLC</div>
                <div style={{ display: "flex", gap: "16px", fontSize: "11px" }}>
                  <span>O: <strong style={{ color: "var(--text-secondary)" }}>{formatPrice(pos.exit_record.trigger_session_ohlc.open)}</strong></span>
                  <span>H: <strong style={{ color: "#10b981" }}>{formatPrice(pos.exit_record.trigger_session_ohlc.high)}</strong></span>
                  <span>L: <strong style={{ color: "#ef4444" }}>{formatPrice(pos.exit_record.trigger_session_ohlc.low)}</strong></span>
                  <span>C: <strong style={{ color: "var(--text-secondary)" }}>{formatPrice(pos.exit_record.trigger_session_ohlc.close)}</strong></span>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default async function LivePage() {
  const { ledger } = await getData();
  const market = getMarketStatus();

  const byState = {
    ACTIVE: ledger.positions.filter((p) => p.lifecycle_state === "ACTIVE"),
    READY_TO_ENTER: ledger.positions.filter((p) => p.lifecycle_state === "READY_TO_ENTER"),
    DECISION_ONLY: ledger.positions.filter((p) => p.lifecycle_state === "DECISION_ONLY"),
    EXITED: ledger.positions.filter((p) => p.lifecycle_state === "EXITED"),
  };

  return (
    <div style={{ maxWidth: "1200px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Header */}
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", flexWrap: "wrap", gap: "16px", marginBottom: "28px" }}>
        <div>
          <p style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>ChronoSentiment</p>
          <h1 style={{ fontSize: "24px", fontWeight: "700", color: "var(--text-primary)", letterSpacing: "-0.02em", margin: "0 0 6px 0" }}>Live Operating Console</h1>
          <p style={{ fontSize: "13px", color: "var(--text-secondary)", margin: 0 }}>NSE · Paper / Research · Decisions sealed at T</p>
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
      <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "12px", marginBottom: "28px" }}>
        {[
          { label: "Active", value: byState.ACTIVE.length, color: "#3b82f6" },
          { label: "Ready to Enter", value: byState.READY_TO_ENTER.length, color: "#f59e0b" },
          { label: "Decision Only", value: byState.DECISION_ONLY.length, color: "#6b7280" },
          { label: "Exited", value: byState.EXITED.length, color: "#10b981" },
        ].map((s) => (
          <div key={s.label} style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "10px", padding: "14px 16px" }}>
            <div style={{ fontSize: "10px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "6px" }}>{s.label}</div>
            <div style={{ fontSize: "24px", fontWeight: "700", color: s.color }}>{s.value}</div>
          </div>
        ))}
      </div>

      {/* Active positions */}
      {byState.ACTIVE.length > 0 && (
        <div style={{ marginBottom: "24px" }}>
          <h2 style={{ fontSize: "13px", fontWeight: "600", color: "#3b82f6", letterSpacing: "0.06em", textTransform: "uppercase", margin: "0 0 12px 0" }}>Active Positions</h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))", gap: "12px" }}>
            {byState.ACTIVE.map((p) => <PositionCard key={p.position_id} pos={p} />)}
          </div>
        </div>
      )}

      {/* Ready to enter */}
      {byState.READY_TO_ENTER.length > 0 && (
        <div style={{ marginBottom: "24px" }}>
          <h2 style={{ fontSize: "13px", fontWeight: "600", color: "#f59e0b", letterSpacing: "0.06em", textTransform: "uppercase", margin: "0 0 12px 0" }}>Ready to Enter</h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))", gap: "12px" }}>
            {byState.READY_TO_ENTER.map((p) => <PositionCard key={p.position_id} pos={p} />)}
          </div>
        </div>
      )}

      {/* Decision only */}
      {byState.DECISION_ONLY.length > 0 && (
        <div style={{ marginBottom: "24px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "12px" }}>
            <h2 style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.06em", textTransform: "uppercase", margin: 0 }}>Decision Only — Awaiting Execution</h2>
            <span style={{ fontSize: "10px", fontWeight: "700", color: "#f59e0b", background: "rgba(245,158,11,0.1)", padding: "2px 6px", borderRadius: "3px" }}>
              Next session: {ledger.positions.find(p => p.next_eligible_session)?.next_eligible_session ?? "—"}
            </span>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))", gap: "12px" }}>
            {byState.DECISION_ONLY.map((p) => <PositionCard key={p.position_id} pos={p} />)}
          </div>
        </div>
      )}

      {/* Exited */}
      {byState.EXITED.length > 0 && (
        <div style={{ marginBottom: "24px" }}>
          <h2 style={{ fontSize: "13px", fontWeight: "600", color: "#10b981", letterSpacing: "0.06em", textTransform: "uppercase", margin: "0 0 12px 0" }}>Exited</h2>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(340px, 1fr))", gap: "12px" }}>
            {byState.EXITED.map((p) => <PositionCard key={p.position_id} pos={p} />)}
          </div>
        </div>
      )}

      {/* Execution architecture */}
      <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "12px", padding: "20px", marginTop: "8px" }}>
        <h2 style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.06em", textTransform: "uppercase", margin: "0 0 16px 0" }}>Execution Architecture</h2>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "12px" }}>
          {[
            { label: "P.E.2 Control", status: "ACTIVE", color: "#10b981", items: ["C3-002 direction", "+5% fixed target", "No risk boundary", "20-session horizon"], note: "Validated. Historical replay PASS." },
            { label: "P.E.3 Treatment", status: "PLANNED", color: "#f59e0b", items: ["C3-002 direction", "Coralys target", "Coralys risk boundary", "20-session horizon"], note: "Specified. Not yet implemented." },
            { label: "Adaptive Execution", status: "FUTURE", color: "#6b7280", items: ["C3-002 direction", "Coralys entry profile", "Adaptive risk boundary", "Per-session reassessment"], note: "Requires P.E.3 evidence first." },
          ].map((exp) => (
            <div key={exp.label} style={{ background: "var(--bg-secondary)", border: "1px solid var(--border-subtle)", borderRadius: "8px", padding: "14px" }}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
                <div style={{ fontSize: "12px", fontWeight: "700", color: "var(--text-primary)" }}>{exp.label}</div>
                <span style={{ fontSize: "10px", fontWeight: "700", color: exp.color, background: `${exp.color}18`, padding: "2px 6px", borderRadius: "3px" }}>{exp.status}</span>
              </div>
              {exp.items.map((item) => (
                <div key={item} style={{ display: "flex", alignItems: "center", gap: "6px", marginBottom: "3px" }}>
                  <div style={{ width: "3px", height: "3px", borderRadius: "50%", background: exp.color, flexShrink: 0 }} />
                  <span style={{ fontSize: "11px", color: "var(--text-secondary)" }}>{item}</span>
                </div>
              ))}
              <div style={{ marginTop: "8px", fontSize: "10px", color: "var(--text-muted)", fontStyle: "italic" }}>{exp.note}</div>
            </div>
          ))}
        </div>
        <div style={{ marginTop: "12px", padding: "10px 14px", background: "rgba(107,114,128,0.06)", border: "1px solid rgba(107,114,128,0.12)", borderRadius: "6px", fontSize: "11px", color: "var(--text-muted)" }}>
          <strong style={{ color: "var(--text-secondary)" }}>Research boundary:</strong> P.E.3 and Adaptive Execution are not yet implemented. The +5% target in P.E.2 is a fixed validation control, not a learned or optimized value. Coralys-derived execution requires a separate experiment with its own evidence before becoming the default.
        </div>
      </div>
    </div>
  );
}