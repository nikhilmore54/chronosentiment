"use client";

/**
 * OutcomeRecorder — Prospective Observation Recorder (Outcome layer)
 *
 * Allows the user to record what happened to a decision after the observation
 * boundary has passed:
 *   - TARGET          → price reached the indicative target
 *   - REFERENCE_RISK  → price hit the reference risk boundary
 *   - HORIZON         → the observation horizon has passed
 *   - USER_CLOSED     → user manually closed the position
 *
 * The user must explicitly confirm that the observation boundary has passed.
 * This is a hard invariant enforced by the backend (AC-O1).
 *
 * POST /api/decisions/{id}/outcome
 * Body: { status, observation_boundary_passed: true, exit_price?, exit_reason? }
 */

import { useState } from "react";

interface OutcomeRecorderProps {
  decisionId: string;
  currentOutcomeStatus: string; // OutcomeStatus from the server
}

type OutcomeStatus = "TARGET" | "REFERENCE_RISK" | "HORIZON" | "USER_CLOSED";
type Phase = "idle" | "submitting" | "done" | "error";

const OUTCOME_OPTIONS: {
  status: OutcomeStatus;
  label: string;
  description: string;
  color: string;
}[] = [
  {
    status: "TARGET",
    label: "Target reached",
    description: "Price reached the indicative target level.",
    color: "#10b981",
  },
  {
    status: "REFERENCE_RISK",
    label: "Reference risk hit",
    description: "Price hit the reference risk boundary.",
    color: "#ef4444",
  },
  {
    status: "HORIZON",
    label: "Horizon passed",
    description: "The observation horizon has elapsed.",
    color: "#6b7280",
  },
  {
    status: "USER_CLOSED",
    label: "Manually closed",
    description: "Position closed manually before any boundary.",
    color: "#f59e0b",
  },
];

export default function OutcomeRecorder({
  decisionId,
  currentOutcomeStatus,
}: OutcomeRecorderProps) {
  const [selected, setSelected] = useState<OutcomeStatus | null>(null);
  const [exitPrice, setExitPrice] = useState<string>("");
  const [boundaryConfirmed, setBoundaryConfirmed] = useState(false);
  const [phase, setPhase] = useState<Phase>("idle");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  // If already closed, show the recorded outcome.
  const alreadyClosed = currentOutcomeStatus !== "OPEN";

  async function submit() {
    if (!selected || !boundaryConfirmed) return;
    setPhase("submitting");
    setErrorMsg(null);

    const parsedPrice = exitPrice.trim() ? parseFloat(exitPrice) : undefined;

    try {
      const res = await fetch(
        `/api/decisions/${encodeURIComponent(decisionId)}/outcome`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            status: selected,
            observation_boundary_passed: true,
            exit_timestamp: new Date().toISOString(),
            exit_price: parsedPrice ?? null,
            exit_reason: null,
            realized_pnl: null,
          }),
        }
      );

      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        setErrorMsg(
          (body as { error?: string }).error ?? `Server error ${res.status}`
        );
        setPhase("error");
        return;
      }

      setPhase("done");
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : "Network error");
      setPhase("error");
    }
  }

  if (alreadyClosed || phase === "done") {
    const label =
      phase === "done"
        ? (OUTCOME_OPTIONS.find((o) => o.status === selected)?.label ?? selected)
        : currentOutcomeStatus.replace(/_/g, " ");
    return (
      <div
        style={{
          marginTop: "24px",
          padding: "12px 16px",
          background: "rgba(16,185,129,0.06)",
          border: "1px solid rgba(16,185,129,0.15)",
          borderRadius: "6px",
          fontSize: "13px",
          fontWeight: "600",
          color: "#10b981",
          textAlign: "center",
        }}
      >
        ✓ Outcome recorded: {label}
      </div>
    );
  }

  return (
    <div
      style={{
        marginTop: "24px",
        padding: "16px",
        background: "var(--bg-card)",
        border: "1px solid var(--border)",
        borderRadius: "8px",
      }}
    >
      <div
        style={{
          fontSize: "10px",
          fontWeight: "600",
          color: "var(--text-muted)",
          letterSpacing: "0.08em",
          textTransform: "uppercase",
          marginBottom: "12px",
        }}
      >
        Record Outcome
      </div>

      {/* Outcome options */}
      <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginBottom: "16px" }}>
        {OUTCOME_OPTIONS.map((opt) => (
          <label
            key={opt.status}
            style={{
              display: "flex",
              alignItems: "flex-start",
              gap: "10px",
              padding: "10px 12px",
              borderRadius: "6px",
              border: `1px solid ${selected === opt.status ? opt.color : "var(--border)"}`,
              background:
                selected === opt.status
                  ? `${opt.color}10`
                  : "var(--bg-surface)",
              cursor: "pointer",
            }}
          >
            <input
              type="radio"
              name="outcome"
              value={opt.status}
              checked={selected === opt.status}
              onChange={() => setSelected(opt.status)}
              style={{ marginTop: "2px", accentColor: opt.color }}
            />
            <div>
              <div
                style={{
                  fontSize: "13px",
                  fontWeight: "600",
                  color: selected === opt.status ? opt.color : "var(--text-primary)",
                }}
              >
                {opt.label}
              </div>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                {opt.description}
              </div>
            </div>
          </label>
        ))}
      </div>

      {/* Optional exit price */}
      <div style={{ marginBottom: "12px" }}>
        <label
          style={{
            display: "block",
            fontSize: "11px",
            color: "var(--text-muted)",
            marginBottom: "4px",
          }}
        >
          Exit price (optional, ₹)
        </label>
        <input
          type="number"
          step="0.01"
          min="0"
          placeholder="e.g. 4850.00"
          value={exitPrice}
          onChange={(e) => setExitPrice(e.target.value)}
          style={{
            width: "100%",
            padding: "8px 10px",
            background: "var(--bg-surface)",
            border: "1px solid var(--border)",
            borderRadius: "5px",
            fontSize: "13px",
            color: "var(--text-primary)",
            boxSizing: "border-box",
          }}
        />
      </div>

      {/* Boundary confirmation */}
      <label
        style={{
          display: "flex",
          alignItems: "flex-start",
          gap: "8px",
          marginBottom: "16px",
          cursor: "pointer",
        }}
      >
        <input
          type="checkbox"
          checked={boundaryConfirmed}
          onChange={(e) => setBoundaryConfirmed(e.target.checked)}
          style={{ marginTop: "2px", accentColor: "#10b981" }}
        />
        <span style={{ fontSize: "11px", color: "var(--text-muted)", lineHeight: "1.5" }}>
          I confirm the observation boundary has passed for this decision.
        </span>
      </label>

      {/* Error */}
      {phase === "error" && errorMsg && (
        <p style={{ fontSize: "11px", color: "#ef4444", marginBottom: "8px" }}>
          {errorMsg}
        </p>
      )}

      {/* Submit */}
      <button
        disabled={!selected || !boundaryConfirmed || phase === "submitting"}
        onClick={submit}
        style={{
          width: "100%",
          padding: "10px 16px",
          background:
            !selected || !boundaryConfirmed || phase === "submitting"
              ? "var(--bg-surface)"
              : "rgba(16,185,129,0.1)",
          border: `1px solid ${
            !selected || !boundaryConfirmed || phase === "submitting"
              ? "var(--border)"
              : "rgba(16,185,129,0.3)"
          }`,
          borderRadius: "6px",
          fontSize: "13px",
          fontWeight: "600",
          color:
            !selected || !boundaryConfirmed || phase === "submitting"
              ? "var(--text-muted)"
              : "#10b981",
          cursor:
            !selected || !boundaryConfirmed || phase === "submitting"
              ? "not-allowed"
              : "pointer",
        }}
      >
        {phase === "submitting" ? "Recording…" : "Record Outcome"}
      </button>
    </div>
  );
}