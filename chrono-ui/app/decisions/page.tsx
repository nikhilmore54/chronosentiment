/**
 * Screen 1 — Decision Feed
 *
 * Displays certified Coralys decisions from the Decision Ledger, newest first.
 * No confidence, ranking, probability, or allocation fields are shown.
 *
 * Data source: coralys_decision_server GET /decisions
 */

import Link from "next/link";
import {
  fetchDecisionFeed,
  formatDecisionTime,
  formatDecisionDate,
  formatPrice,
  directionLabel,
  type FeedEntry,
} from "@/lib/coralys";

// ─── Direction badge ──────────────────────────────────────────────────────────

function DirectionBadge({ direction }: { direction: string }) {
  const colors: Record<string, { bg: string; color: string; border: string }> = {
    LONG: {
      bg: "rgba(16,185,129,0.08)",
      color: "#10b981",
      border: "rgba(16,185,129,0.2)",
    },
    SHORT: {
      bg: "rgba(239,68,68,0.08)",
      color: "#ef4444",
      border: "rgba(239,68,68,0.2)",
    },
    NO_TRADE: {
      bg: "rgba(107,114,128,0.08)",
      color: "#6b7280",
      border: "rgba(107,114,128,0.2)",
    },
  };
  const c = colors[direction] ?? colors.NO_TRADE;
  return (
    <span
      style={{
        padding: "2px 8px",
        background: c.bg,
        border: `1px solid ${c.border}`,
        borderRadius: "4px",
        fontSize: "11px",
        fontWeight: "700",
        color: c.color,
        letterSpacing: "0.04em",
      }}
    >
      {directionLabel(direction as "LONG" | "SHORT" | "NO_TRADE")}
    </span>
  );
}

// ─── Execution status badge ───────────────────────────────────────────────────

function ExecutionBadge({ status }: { status: string }) {
  if (status === "UserExecuted") {
    return (
      <span style={{ fontSize: "11px", color: "#10b981", fontWeight: "600" }}>
        Executed
      </span>
    );
  }
  if (status === "UserIgnored") {
    return (
      <span style={{ fontSize: "11px", color: "#6b7280" }}>Ignored</span>
    );
  }
  return (
    <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>—</span>
  );
}

// ─── Outcome badge ────────────────────────────────────────────────────────────

function OutcomeBadge({ status }: { status: string }) {
  if (status === "Open") {
    return (
      <span
        style={{
          padding: "2px 6px",
          background: "rgba(245,158,11,0.08)",
          border: "1px solid rgba(245,158,11,0.2)",
          borderRadius: "3px",
          fontSize: "10px",
          fontWeight: "600",
          color: "#f59e0b",
        }}
      >
        OPEN
      </span>
    );
  }
  if (status === "Target") {
    return (
      <span
        style={{
          padding: "2px 6px",
          background: "rgba(16,185,129,0.08)",
          border: "1px solid rgba(16,185,129,0.2)",
          borderRadius: "3px",
          fontSize: "10px",
          fontWeight: "600",
          color: "#10b981",
        }}
      >
        TARGET
      </span>
    );
  }
  if (status === "ReferenceRisk") {
    return (
      <span
        style={{
          padding: "2px 6px",
          background: "rgba(239,68,68,0.08)",
          border: "1px solid rgba(239,68,68,0.2)",
          borderRadius: "3px",
          fontSize: "10px",
          fontWeight: "600",
          color: "#ef4444",
        }}
      >
        REF RISK
      </span>
    );
  }
  return (
    <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>{status}</span>
  );
}

// ─── Decision card ────────────────────────────────────────────────────────────

function DecisionCard({ entry }: { entry: FeedEntry }) {
  return (
    <Link
      href={`/decisions/${entry.decision_id}`}
      style={{ textDecoration: "none" }}
    >
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
        {/* Header row */}
        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "space-between",
            marginBottom: "12px",
          }}
        >
          <div>
            <div
              style={{
                fontSize: "15px",
                fontWeight: "700",
                color: "var(--text-primary)",
              }}
            >
              {entry.instrument}
            </div>
            <div
              style={{
                fontSize: "11px",
                color: "var(--text-muted)",
                marginTop: "2px",
              }}
            >
              {formatDecisionTime(entry.decision_timestamp)}
            </div>
          </div>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "flex-end",
              gap: "4px",
            }}
          >
            <DirectionBadge direction={entry.direction} />
            <span
              style={{
                padding: "2px 6px",
                background: "rgba(16,185,129,0.06)",
                border: "1px solid rgba(16,185,129,0.15)",
                borderRadius: "3px",
                fontSize: "10px",
                fontWeight: "600",
                color: "#10b981",
                letterSpacing: "0.04em",
              }}
            >
              CERTIFIED ✓
            </span>
          </div>
        </div>

        {/* Price row */}
        <div style={{ display: "flex", gap: "24px", marginBottom: "10px" }}>
          <div>
            <div
              style={{
                fontSize: "10px",
                color: "var(--text-muted)",
                marginBottom: "2px",
              }}
            >
              Target
            </div>
            <div
              style={{
                fontSize: "13px",
                fontWeight: "600",
                color: "var(--text-primary)",
              }}
            >
              {formatPrice(entry.target_price)}
            </div>
          </div>
          <div>
            <div
              style={{
                fontSize: "10px",
                color: "var(--text-muted)",
                marginBottom: "2px",
              }}
            >
              Reference Risk
            </div>
            <div
              style={{
                fontSize: "13px",
                fontWeight: "600",
                color: "var(--text-secondary)",
              }}
            >
              {formatPrice(entry.reference_risk_boundary_price)}
            </div>
          </div>
        </div>

        {/* Footer row */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            paddingTop: "10px",
            borderTop: "1px solid var(--border-subtle)",
          }}
        >
          <OutcomeBadge status={entry.outcome_status} />
          <ExecutionBadge status={entry.execution_status} />
        </div>
      </div>
    </Link>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default async function DecisionFeedPage() {
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
          Decision Feed
        </h1>
        <p style={{ fontSize: "13px", color: "var(--text-secondary)", margin: 0 }}>
          {feed.total} certified decision{feed.total !== 1 ? "s" : ""} · newest first
        </p>
      </div>

      {/* Navigation */}
      <div style={{ display: "flex", gap: "12px", marginBottom: "28px" }}>
        <Link
          href="/decisions"
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
          Feed
        </Link>
        <Link
          href="/history"
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
            No certified decisions yet
          </div>
          <div style={{ fontSize: "12px" }}>
            Decisions will appear here once the Coralys pipeline produces and certifies them.
          </div>
        </div>
      )}

      {/* Decision grid */}
      {feed.total > 0 && (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))",
            gap: "10px",
          }}
        >
          {feed.decisions.map((entry) => (
            <DecisionCard key={entry.decision_id} entry={entry} />
          ))}
        </div>
      )}
    </div>
  );
}