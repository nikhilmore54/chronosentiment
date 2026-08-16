import Link from "next/link";
import { promises as fs } from "fs";
import path from "path";
import type { HistoricalLedger } from "@/lib/data";
import { formatDate, shortHash } from "@/lib/data";

async function getData() {
  const dataDir = path.join(process.cwd(), "public", "data");
  const [histRaw, prospRaw] = await Promise.all([
    fs.readFile(path.join(dataDir, "historical_ledger.json"), "utf-8"),
    fs.readFile(path.join(dataDir, "prospective_ledger.json"), "utf-8"),
  ]);
  return {
    hist: JSON.parse(histRaw) as HistoricalLedger,
    prosp: JSON.parse(prospRaw) as HistoricalLedger,
  };
}

export default async function DecisionsPage() {
  const { hist, prosp } = await getData();

  // Merge all decisions with source tag
  const allDecisions = [
    ...prosp.decisions.map((d) => ({ ...d, source: "prospective" as const, obs: null })),
    ...hist.decisions.map((d) => ({
      ...d,
      source: "historical" as const,
      obs: hist.observations.find((o) => o.decision_id === d.decision_id) ?? null,
    })),
  ];

  const instruments = Array.from(new Set(allDecisions.map((d) => d.instrument))).sort();

  return (
    <div style={{ maxWidth: "1200px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Header */}
      <div style={{ marginBottom: "32px" }}>
        <p style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", letterSpacing: "0.1em", textTransform: "uppercase", marginBottom: "8px" }}>
          Observatory
        </p>
        <h1 style={{ fontSize: "24px", fontWeight: "700", color: "var(--text-primary)", letterSpacing: "-0.02em", margin: "0 0 8px 0" }}>
          Decision Feed
        </h1>
        <p style={{ fontSize: "13px", color: "var(--text-secondary)", margin: 0 }}>
          {prosp.decisions.length} observing · {hist.observations.length} completed evidence · {allDecisions.length} total
        </p>
      </div>

      {/* Instrument filter pills */}
      <div style={{ display: "flex", gap: "8px", flexWrap: "wrap", marginBottom: "24px" }}>
        {instruments.map((inst) => (
          <div
            key={inst}
            style={{
              padding: "4px 12px",
              background: "var(--bg-card)",
              border: "1px solid var(--border)",
              borderRadius: "20px",
              fontSize: "12px",
              color: "var(--text-secondary)",
              fontWeight: "500",
            }}
          >
            {inst}
          </div>
        ))}
      </div>

      {/* Live cohort section */}
      <div style={{ marginBottom: "40px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "16px" }}>
          <h2 style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)", margin: 0 }}>
            Live Cohort — 14 August 2026
          </h2>
          <span className="badge badge-observing">7 Observing</span>
          <div
            style={{
              padding: "3px 8px",
              background: "rgba(245, 158, 11, 0.08)",
              border: "1px solid rgba(245, 158, 11, 0.15)",
              borderRadius: "4px",
              fontSize: "10px",
              color: "#f59e0b",
              fontWeight: "600",
              letterSpacing: "0.05em",
            }}
          >
            DECISION-ONLY · NO EXECUTION INTENT
          </div>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))", gap: "10px" }}>
          {prosp.decisions.map((d) => (
            <Link key={d.decision_id} href={`/decisions/${d.decision_id}`} style={{ textDecoration: "none" }}>
              <div
                className="card-hover"
                style={{
                  background: "var(--bg-card)",
                  border: "1px solid var(--border)",
                  borderRadius: "8px",
                  padding: "16px",
                  cursor: "pointer",
                }}
              >
                <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", marginBottom: "12px" }}>
                  <div>
                    <div style={{ fontSize: "15px", fontWeight: "700", color: "var(--text-primary)" }}>{d.instrument}</div>
                    <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                      {formatDate(d.decision_time)}
                    </div>
                  </div>
                  <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: "4px" }}>
                    <span className={d.action === "LONG" ? "badge badge-long" : d.action === "SHORT" ? "badge badge-short" : "badge badge-no-trade"}>
                      {d.action}
                    </span>
                    <span className="badge badge-observing">Observing</span>
                  </div>
                </div>
                <div style={{ display: "flex", gap: "16px" }}>
                  <div>
                    <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Market State</div>
                    <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                      {d.state.trend} / {d.state.momentum}
                    </div>
                  </div>
                  <div>
                    <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>Policy</div>
                    <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>{d.policy_id}</div>
                  </div>
                </div>
                <div style={{ marginTop: "10px", paddingTop: "10px", borderTop: "1px solid var(--border-subtle)" }}>
                  <div style={{ fontSize: "10px", color: "var(--text-muted)", fontFamily: "monospace" }}>
                    {shortHash(d.decision_id)}
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {/* Historical evidence section */}
      <div>
        <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "16px" }}>
          <h2 style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)", margin: 0 }}>
            Historical Evidence — 2024
          </h2>
          <span
            style={{
              padding: "2px 8px",
              background: "rgba(16,185,129,0.1)",
              border: "1px solid rgba(16,185,129,0.2)",
              borderRadius: "4px",
              fontSize: "11px",
              fontWeight: "600",
              color: "#10b981",
            }}
          >
            {hist.observations.length} Completed
          </span>
        </div>

        <div style={{ background: "var(--bg-card)", border: "1px solid var(--border)", borderRadius: "8px", overflow: "hidden" }}>
          {/* Table header */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 120px 100px 100px 120px 100px",
              padding: "10px 16px",
              borderBottom: "1px solid var(--border)",
              fontSize: "10px",
              fontWeight: "600",
              color: "var(--text-muted)",
              letterSpacing: "0.05em",
              textTransform: "uppercase",
            }}
          >
            <div>Instrument</div>
            <div>Decision Date</div>
            <div>Direction</div>
            <div>State</div>
            <div>Evidence</div>
            <div style={{ textAlign: "right" }}>Return</div>
          </div>

          {/* Table rows */}
          {hist.decisions.map((d, i) => {
            const obs = hist.observations.find((o) => o.decision_id === d.decision_id);
            const ret = obs ? (d.action === "LONG" ? obs.value_long : obs.value_short) : null;
            const positive = ret !== null && ret >= 0;
            return (
              <Link key={d.decision_id} href={`/decisions/${d.decision_id}`} style={{ textDecoration: "none" }}>
                <div
                  className="card-hover"
                  style={{
                    display: "grid",
                    gridTemplateColumns: "1fr 120px 100px 100px 120px 100px",
                    padding: "10px 16px",
                    borderBottom: i < hist.decisions.length - 1 ? "1px solid var(--border-subtle)" : "none",
                    cursor: "pointer",
                    alignItems: "center",
                  }}
                >
                  <div style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)" }}>
                    {d.instrument}
                  </div>
                  <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                    {formatDate(d.decision_time)}
                  </div>
                  <div>
                    <span className={d.action === "LONG" ? "badge badge-long" : d.action === "SHORT" ? "badge badge-short" : "badge badge-no-trade"}>
                      {d.action}
                    </span>
                  </div>
                  <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                    {d.state.trend}
                  </div>
                  <div>
                    {obs ? (
                      <span
                        style={{
                          fontSize: "11px",
                          fontWeight: "600",
                          color: "#10b981",
                          background: "rgba(16,185,129,0.08)",
                          padding: "2px 6px",
                          borderRadius: "3px",
                        }}
                      >
                        Completed
                      </span>
                    ) : (
                      <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>Pending</span>
                    )}
                  </div>
                  <div style={{ textAlign: "right", fontSize: "13px", fontWeight: "600", color: ret !== null ? (positive ? "#10b981" : "#ef4444") : "var(--text-muted)" }}>
                    {ret !== null ? ((ret >= 0 ? "+" : "") + (ret * 100).toFixed(2) + "%") : "—"}
                  </div>
                </div>
              </Link>
            );
          })}
        </div>
      </div>
    </div>
  );
}