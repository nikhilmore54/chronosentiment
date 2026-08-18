/**
 * Screen 2 — Decision Detail
 *
 * Displays the complete certified DecisionRecord for a single decision.
 * Shows all five sections: Identity, Certification, Decision, Reference Risk,
 * Execution, Outcome, Evidence.
 *
 * Data source: coralys_decision_server GET /decisions/{id}
 */

import Link from "next/link";
import { notFound } from "next/navigation";
import {
  fetchDecision,
  formatDecisionTime,
  formatDecisionDate,
  formatPrice,
  formatPct,
  directionLabel,
  shortHash,
  computeIndicativePrices,
  type CoralysDecision,
} from "@/lib/coralys";

// ─── Section wrapper ──────────────────────────────────────────────────────────

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div style={{ marginBottom: "28px" }}>
      <div
        style={{
          fontSize: "10px",
          fontWeight: "600",
          color: "var(--text-muted)",
          letterSpacing: "0.08em",
          textTransform: "uppercase",
          marginBottom: "12px",
          paddingBottom: "8px",
          borderBottom: "1px solid var(--border-subtle)",
        }}
      >
        {title}
      </div>
      {children}
    </div>
  );
}

// ─── Field row ────────────────────────────────────────────────────────────────

function Field({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "baseline",
        padding: "6px 0",
        borderBottom: "1px solid var(--border-subtle)",
      }}
    >
      <span
        style={{
          fontSize: "12px",
          color: "var(--text-muted)",
          minWidth: "160px",
        }}
      >
        {label}
      </span>
      <span
        style={{
          fontSize: "13px",
          color: "var(--text-primary)",
          fontWeight: "500",
          textAlign: "right",
        }}
      >
        {value}
      </span>
    </div>
  );
}

// ─── Direction badge ──────────────────────────────────────────────────────────

function DirectionBadge({ direction }: { direction: string }) {
  const colors: Record<string, { bg: string; color: string; border: string }> = {
    LONG: { bg: "rgba(16,185,129,0.08)", color: "#10b981", border: "rgba(16,185,129,0.2)" },
    SHORT: { bg: "rgba(239,68,68,0.08)", color: "#ef4444", border: "rgba(239,68,68,0.2)" },
    NO_TRADE: { bg: "rgba(107,114,128,0.08)", color: "#6b7280", border: "rgba(107,114,128,0.2)" },
  };
  const c = colors[direction] ?? colors.NO_TRADE;
  return (
    <span
      style={{
        padding: "3px 10px",
        background: c.bg,
        border: `1px solid ${c.border}`,
        borderRadius: "4px",
        fontSize: "13px",
        fontWeight: "700",
        color: c.color,
        letterSpacing: "0.04em",
      }}
    >
      {directionLabel(direction as "LONG" | "SHORT" | "NO_TRADE")}
    </span>
  );
}

// ─── Evidence row ─────────────────────────────────────────────────────────────

function EvidenceField({
  label,
  value,
}: {
  label: string;
  value: number | null;
  format?: (n: number) => string;
}) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "baseline",
        padding: "6px 0",
        borderBottom: "1px solid var(--border-subtle)",
      }}
    >
      <span style={{ fontSize: "12px", color: "var(--text-muted)", minWidth: "200px" }}>
        {label}
      </span>
      <span
        style={{
          fontSize: "13px",
          color: value !== null ? "var(--text-primary)" : "var(--text-muted)",
          fontWeight: value !== null ? "500" : "400",
          fontStyle: value === null ? "italic" : "normal",
        }}
      >
        {value !== null ? String(value) : "Awaiting prospective observation"}
      </span>
    </div>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default async function DecisionDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const decision = await fetchDecision(id);

  if (!decision) {
    notFound();
  }

  const d = decision as CoralysDecision;

  const indicative = computeIndicativePrices(
    d.decision.reference_price ?? null,
    d.decision.atr_14 ?? null,
    d.decision.trend,
    d.decision.momentum,
    d.decision.direction,
  );

  return (
    <div style={{ maxWidth: "720px", margin: "0 auto", padding: "32px 24px" }}>
      {/* Back link */}
      <div style={{ marginBottom: "24px" }}>
        <Link
          href="/decisions"
          style={{
            fontSize: "12px",
            color: "var(--text-muted)",
            textDecoration: "none",
            display: "inline-flex",
            alignItems: "center",
            gap: "4px",
          }}
        >
          ← Decision Feed
        </Link>
      </div>

      {/* Hero */}
      <div style={{ marginBottom: "32px" }}>
        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "space-between",
            marginBottom: "8px",
          }}
        >
          <div>
            <h1
              style={{
                fontSize: "28px",
                fontWeight: "700",
                color: "var(--text-primary)",
                letterSpacing: "-0.02em",
                margin: "0 0 4px 0",
              }}
            >
              {d.identity.instrument}
            </h1>
            <div style={{ fontSize: "13px", color: "var(--text-secondary)" }}>
              {formatDecisionDate(d.identity.decision_timestamp)} ·{" "}
              {formatDecisionTime(d.identity.decision_timestamp)}
            </div>
          </div>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "flex-end",
              gap: "6px",
            }}
          >
            <DirectionBadge direction={d.decision.direction} />
            <span
              style={{
                padding: "2px 8px",
                background: "rgba(16,185,129,0.06)",
                border: "1px solid rgba(16,185,129,0.15)",
                borderRadius: "3px",
                fontSize: "11px",
                fontWeight: "600",
                color: "#10b981",
                letterSpacing: "0.04em",
              }}
            >
              CERTIFIED ✓
            </span>
          </div>
        </div>
      </div>

      {/* Decision */}
      <Section title="Decision">
        <Field label="Trend" value={d.decision.trend} />
        <Field label="Momentum" value={d.decision.momentum} />
        <Field label="Volatility" value={d.decision.volatility} />
        {d.decision.effective_session && (
          <Field label="For session" value={d.decision.effective_session} />
        )}
        <Field
          label="Reference Price"
          value={
            <span style={{ fontSize: "15px", fontWeight: "700", color: "var(--text-primary)" }}>
              {formatPrice(d.decision.reference_price ?? null)}
            </span>
          }
        />
        {d.decision.atr_14 && (
          <Field label="ATR-14" value={formatPrice(d.decision.atr_14)} />
        )}
        <Field
          label="Indicative Target"
          value={
            indicative ? (
              <span style={{ fontSize: "15px", fontWeight: "700", color: "#10b981" }}>
                {formatPrice(indicative.indicative_target)}{" "}
                <span style={{ fontSize: "11px", fontWeight: "400", color: "#10b981" }}>
                  {d.decision.reference_price != null
                    ? formatPct((indicative.indicative_target - d.decision.reference_price) / d.decision.reference_price)
                    : ""}
                </span>
              </span>
            ) : (
              <span style={{ color: "var(--text-muted)", fontStyle: "italic" }}>
                Awaiting execution
              </span>
            )
          }
        />
        <Field
          label="Indicative Risk"
          value={
            indicative ? (
              <span style={{ fontSize: "15px", fontWeight: "700", color: "#f59e0b" }}>
                {formatPrice(indicative.indicative_risk)}{" "}
                <span style={{ fontSize: "11px", fontWeight: "400", color: "#f59e0b" }}>
                  {d.decision.reference_price != null
                    ? formatPct((indicative.indicative_risk - d.decision.reference_price) / d.decision.reference_price)
                    : ""}
                </span>
              </span>
            ) : (
              <span style={{ color: "var(--text-muted)", fontStyle: "italic" }}>
                Awaiting execution
              </span>
            )
          }
        />
        <Field
          label="Target (sealed)"
          value={
            <span style={{ color: "var(--text-muted)", fontStyle: "italic" }}>
              {d.decision.target_price !== null ? formatPrice(d.decision.target_price) : "Set at execution"}
            </span>
          }
        />
      </Section>

      {/* Reference Risk */}
      <Section title="Reference Risk">
        <Field
          label="Boundary"
          value={
            d.reference_risk.boundary_price !== null ? (
              <span style={{ fontSize: "16px", fontWeight: "700", color: "#f59e0b" }}>
                {formatPrice(d.reference_risk.boundary_price)}
              </span>
            ) : (
              <span style={{ color: "var(--text-muted)", fontStyle: "italic", fontSize: "13px" }}>
                Not recorded at certification
              </span>
            )
          }
        />
        <Field label="Type" value={d.reference_risk.boundary_type} />
        <Field label="Status" value="REFERENCE" />
      </Section>

      {/* Historical Evidence */}
      <Section title="Historical Evidence">
        <EvidenceField
          label="Similar decisions"
          value={d.evidence.similar_decisions_count}
        />
        <EvidenceField
          label="Historical target rate"
          value={d.evidence.historical_target_rate}
        />
        <EvidenceField
          label="Median MAE %"
          value={d.evidence.median_mae_pct}
        />
        <EvidenceField
          label="P90 MAE %"
          value={d.evidence.p90_mae_pct}
        />
        <EvidenceField
          label="Median MFE %"
          value={d.evidence.median_mfe_pct}
        />
        <EvidenceField
          label="Median sessions to target"
          value={d.evidence.median_time_to_target_sessions}
        />
      </Section>

      {/* Execution */}
      <Section title="Execution">
        <Field label="Status" value={d.execution.status.replace(/_/g, " ")} />
        {d.execution.execution_timestamp && (
          <Field
            label="Executed at"
            value={formatDecisionTime(d.execution.execution_timestamp)}
          />
        )}
        {d.execution.execution_price !== null && (
          <Field label="Execution price" value={formatPrice(d.execution.execution_price)} />
        )}
        {d.execution.quantity !== null && (
          <Field label="Quantity" value={String(d.execution.quantity)} />
        )}
      </Section>

      {/* Outcome */}
      <Section title="Outcome">
        <Field label="Status" value={d.outcome.status} />
        {d.outcome.exit_price !== null && (
          <Field label="Exit price" value={formatPrice(d.outcome.exit_price)} />
        )}
        {d.outcome.realized_pnl !== null && (
          <Field
            label="Realized P&L"
            value={
              <span
                style={{
                  color: d.outcome.realized_pnl >= 0 ? "#10b981" : "#ef4444",
                  fontWeight: "700",
                }}
              >
                {d.outcome.realized_pnl >= 0 ? "+" : ""}
                {formatPrice(d.outcome.realized_pnl)}
              </span>
            }
          />
        )}
      </Section>

      {/* Certification / Provenance */}
      <Section title="Certification">
        <Field label="Pipeline" value={d.certification.decision_pipeline} />
        <Field
          label="Certified at"
          value={formatDecisionTime(d.certification.certified_timestamp)}
        />
        <Field
          label="Policy artifact"
          value={
            <span style={{ fontFamily: "monospace", fontSize: "11px" }}>
              {d.certification.policy_artifact_hash.slice(0, 16)}…
            </span>
          }
        />
        {d.certification.execution_artifact_hash && (
          <Field
            label="Execution artifact"
            value={
              <span style={{ fontFamily: "monospace", fontSize: "11px" }}>
                {d.certification.execution_artifact_hash.slice(0, 16)}…
              </span>
            }
          />
        )}
        <Field
          label="Data snapshot"
          value={
            <span style={{ fontFamily: "monospace", fontSize: "11px" }}>
              {d.certification.data_snapshot_id}
            </span>
          }
        />
        <Field
          label="Decision ID"
          value={
            <span style={{ fontFamily: "monospace", fontSize: "11px" }}>
              {shortHash(d.identity.decision_id)}
            </span>
          }
        />
      </Section>

      {/* Action buttons — MVP-007/008 will wire these up */}
      <div
        style={{
          display: "flex",
          gap: "12px",
          marginTop: "32px",
          paddingTop: "24px",
          borderTop: "1px solid var(--border)",
        }}
      >
        <button
          disabled
          style={{
            flex: 1,
            padding: "12px 16px",
            background: "rgba(16,185,129,0.08)",
            border: "1px solid rgba(16,185,129,0.2)",
            borderRadius: "6px",
            fontSize: "13px",
            fontWeight: "600",
            color: "#10b981",
            cursor: "not-allowed",
            opacity: 0.6,
          }}
        >
          I EXECUTED THIS
        </button>
        <button
          disabled
          style={{
            flex: 1,
            padding: "12px 16px",
            background: "var(--bg-card)",
            border: "1px solid var(--border)",
            borderRadius: "6px",
            fontSize: "13px",
            fontWeight: "600",
            color: "var(--text-secondary)",
            cursor: "not-allowed",
            opacity: 0.6,
          }}
        >
          IGNORE
        </button>
      </div>
      <p
        style={{
          fontSize: "11px",
          color: "var(--text-muted)",
          textAlign: "center",
          marginTop: "8px",
        }}
      >
        Execution is user-controlled. Recording an action does not place an order.
      </p>
    </div>
  );
}
