/**
 * Screen 3 — Decision History
 *
 * Tabular view of all certified decisions: date, instrument, direction,
 * user action, and outcome. This is the observatory of decisions — it shows
 * what Coralys decided, what the user did, and what happened.
 *
 * Data source: coralys_decision_server GET /decisions
 */

import Link from "next/link";
import {
  fetchDecisionFeed,
  formatDecisionDate,
  formatDecisionTime,
  directionLabel,
  type FeedEntry,
} from "@/lib/coralys";

// ─── Direction cell ───────────────────────────────────────────────────────────

function DirectionCell({ direction }: { direction: string }) {
  const colors: Record<string, string> = {
    LONG: "#10b981",
    SHORT: "#ef4444",
    NO_TRADE: "#6b7280",
  };
  return (
    <span
      style={{
        fontSize: "12px",
        fontWeight: "700",
        color: colors[direction] ?? "#6b7280",
      }}
    >
      {directionLabel(direction as "LONG" | "SHORT" | "NO_TRADE")}
    </span>
  );
}

// ─── User action cell ─────────────────────────────────────────────────────────

function UserActionCell({ status }: { status: string }) {
  if (status === "UserExecuted") {
    return (
      <span
        style={{
          padding: "2px 6px",
          background: "rgba(16,185,129,0.08)",
          border: "1px solid rgba(16,185,129,0.2)",
          borderRadius: "3px",
          fontSize: "11px",
          fontWeight: "600",
          color: "#10b981",
        }}
      >
        Executed
      </span>
    );
  }
  if (status === "UserIgnored") {
    return (
      <span
        style={{
          padding: "2px 6px",
          background: "rgba(107,114,128,0.06)",
          border: "1px solid rgba(107,114,128,0.15)",
          borderRadius: "3px",
          fontSize: "11px",
          fontWeight: "600",
          color: "#6b7280",
        }}
      >
        Ignored
      </span>
    );
  }
  return (
    <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>—</span>
  );
}

// ─── Outcome cell ─────────────────────────────────────────────────────────────

function OutcomeCell({ status }: { status: string }) {
  const map: Record<string, { label: string; color: string }> = {
    Open: { label: "OPEN", color: "#f59e0b" },
    Target: { label: "TARGET", color: "#10b981" },
    ReferenceRisk: { label: "REF RISK", color: "#ef4444" },
    Horizon: { label: "HORIZON", color: "#6b7280" },
    UserClosed: { label: "CLOSED", color: "#6b7280" },
  };
  const m = map[status];
  if (!m) return <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>{status}</span>;
  return (
    <span style={{ fontSize: "11px", fontWeight: "600", color: m.color }}>
      {m.label}
    </span>
  );
}

// ─── Table row ────────────────────────────────────────────────────────────────

function HistoryRow({ entry, isLast }: { entry: FeedEntry; isLast: boolean }) {
  return (
    <Link
      href={`/decisions/${entry.decision_id}`}
      style={{ textDecoration: "none" }}
    >
      <div
        className="card-hover"
        style={{
          display: "grid",
          gridTemplateColumns: "140px 1fr 80px 120px 100px",
          padding: "10px 16px",
          borderBottom: isLast ? "none" : "1px solid var(--border-subtle)",
          cursor: "pointer",
          alignItems: "center",
        }}
      >
        {/* Date + time */}
        <div>
          <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
            {formatDecisionDate(entry.decision_timestamp)}
          </div>
          <div style={{ fontSize: "10px", color: "var(--text-muted)", marginTop: "1px" }}>
            {formatDecisionTime(entry.decision_timestamp)}
          </div>
        </div>

        {/* Instrument */}
        <div
          style={{
            fontSize: "13px",
            fontWeight: "600",
            color: "var(--text-primary)",
          }}
        >
          {entry.instrument}
        </div>

        {/* Direction */}
        <div>
          <DirectionCell direction={entry.direction} />
        </div>

        {/* User action */}
        <div>
          <UserActionCell status={entry.execution_status} />
        </div>

        {/* Outcome */}
        <div style={{ textAlign: "right" }}>
          <OutcomeCell status={entry.outcome_status} />
        </div>
      </div>
    </Link>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default async function HistoryPage() {
  const feed = await fetchDecisionFeed();

  return (
    <div style={{ maxWidth: "1200px", margin: "0 auto", padding: "32px 24px" }}>
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
          Coralys Decision Intelligence
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
          Decision History
        </h1>
        <p style={{ fontSize: "13px", color: "var(--text-secondary)", margin: 0 }}>
          {feed.total} certified decision{feed.total !== 1 ? "s" : ""} · complete lifecycle record
        </p>
      </div>

      {/* Navigation */}
      <div style={{ display: "flex", gap: "12px", marginBottom: "28px" }}>
        <Link
          href="/decisions"
          style={{
            fontSize: "12px",
            fontWeight: "500",
            color: "var(--text-secondary)",
            textDecoration: "none",
            padding: "6px 12px",
            background: "transparent",
            border: "1px solid transparent",
            borderRadius: "6px",
          }}
        >
          Feed
        </Link>
        <Link
          href="/history"
          style={{
            fontSize: "12px",
            fontWeight: "600",
            color: "var(--text-primary)",
            textDecoration: "none",
            padding: "6px 12px",
            background: "var(--bg-card)",
            border: "1px solid var(--border)",
            borderRadius: "6px",
          }}
        >
          History
        </Link>
      </div>

      {/* Empty state */}
      {feed.total === 0 && (
        <div
          style={{
            textAlign: "center",
            padding: "64px 24px",
            color: "var(--text-muted)",
          }}
        >
          <div style={{ fontSize: "32px", marginBottom: "16px" }}>—</div>
          <div style={{ fontSize: "14px", fontWeight: "600", marginBottom: "8px" }}>
            No decision history yet
          </div>
          <div style={{ fontSize: "12px" }}>
            Certified decisions will appear here once the Coralys pipeline is running.
          </div>
        </div>
      )}

      {/* History table */}
      {feed.total > 0 && (
        <div
          style={{
            background: "var(--bg-card)",
            border: "1px solid var(--border)",
            borderRadius: "8px",
            overflow: "hidden",
          }}
        >
          {/* Table header */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "140px 1fr 80px 120px 100px",
              padding: "10px 16px",
              borderBottom: "1px solid var(--border)",
              fontSize: "10px",
              fontWeight: "600",
              color: "var(--text-muted)",
              letterSpacing: "0.05em",
              textTransform: "uppercase",
            }}
          >
            <div>Date</div>
            <div>Instrument</div>
            <div>Direction</div>
            <div>User Action</div>
            <div style={{ textAlign: "right" }}>Outcome</div>
          </div>

          {/* Rows */}
          {feed.decisions.map((entry, i) => (
            <HistoryRow
              key={entry.decision_id}
              entry={entry}
              isLast={i === feed.decisions.length - 1}
            />
          ))}
        </div>
      )}

      {/* Legend */}
      {feed.total > 0 && (
        <div
          style={{
            marginTop: "16px",
            display: "flex",
            gap: "16px",
            flexWrap: "wrap",
          }}
        >
          {[
            { label: "OPEN", color: "#f59e0b" },
            { label: "TARGET", color: "#10b981" },
            { label: "REF RISK", color: "#ef4444" },
            { label: "HORIZON", color: "#6b7280" },
          ].map(({ label, color }) => (
            <div
              key={label}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "6px",
                fontSize: "11px",
                color: "var(--text-muted)",
              }}
            >
              <span
                style={{
                  width: "8px",
                  height: "8px",
                  borderRadius: "50%",
                  background: color,
                  display: "inline-block",
                }}
              />
              {label}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}