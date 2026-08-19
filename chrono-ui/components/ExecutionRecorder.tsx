"use client";

/**
 * ExecutionRecorder — Prospective Observation Recorder (Execution layer)
 *
 * Allows the user to record what they did after receiving a certified decision:
 *   - I EXECUTED THIS  → POST /decisions/{id}/execution  { status: "USER_EXECUTED" }
 *   - IGNORE           → POST /decisions/{id}/execution  { status: "USER_IGNORED" }
 *
 * This is a user-reported action. Coralys does not infer quantity, allocation,
 * or capital deployment. Recording an action does not place an order.
 *
 * AC-E1: POST execution for valid decision → 200 with updated record.
 * AC-E3: quantity: null accepted (no quantity inference).
 * AC-E4: No quantity/allocation inference in any code path.
 * AC-E8: USER_IGNORED is a valid execution status.
 */

import { useState } from "react";

interface ExecutionRecorderProps {
  decisionId: string;
  currentStatus: string; // ExecutionStatus from the server
}

type Phase = "idle" | "submitting" | "done" | "error";

export default function ExecutionRecorder({
  decisionId,
  currentStatus,
}: ExecutionRecorderProps) {
  const [phase, setPhase] = useState<Phase>("idle");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [recorded, setRecorded] = useState<string | null>(null);

  // If already recorded, show the recorded status.
  const alreadyRecorded =
    currentStatus === "USER_EXECUTED" ||
    currentStatus === "USER_IGNORED" ||
    currentStatus === "USER_CANCELLED";

  async function submit(status: "USER_EXECUTED" | "USER_IGNORED") {
    setPhase("submitting");
    setErrorMsg(null);
    try {
      const res = await fetch(
        `/api/decisions/${encodeURIComponent(decisionId)}/execution`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            status,
            execution_timestamp: new Date().toISOString(),
            quantity: null,
            execution_price: null,
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
      setRecorded(status === "USER_EXECUTED" ? "Executed" : "Ignored");
      setPhase("done");
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : "Network error");
      setPhase("error");
    }
  }

  if (alreadyRecorded || phase === "done") {
    const label =
      recorded ??
      (currentStatus === "USER_EXECUTED"
        ? "Executed"
        : currentStatus === "USER_IGNORED"
        ? "Ignored"
        : "Cancelled");
    return (
      <div style={{ marginTop: "32px", paddingTop: "24px", borderTop: "1px solid var(--border)" }}>
        <div
          style={{
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
          ✓ Recorded: {label}
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

  return (
    <div style={{ marginTop: "32px", paddingTop: "24px", borderTop: "1px solid var(--border)" }}>
      <div style={{ display: "flex", gap: "12px" }}>
        <button
          disabled={phase === "submitting"}
          onClick={() => submit("USER_EXECUTED")}
          style={{
            flex: 1,
            padding: "12px 16px",
            background: "rgba(16,185,129,0.08)",
            border: "1px solid rgba(16,185,129,0.2)",
            borderRadius: "6px",
            fontSize: "13px",
            fontWeight: "600",
            color: "#10b981",
            cursor: phase === "submitting" ? "not-allowed" : "pointer",
            opacity: phase === "submitting" ? 0.6 : 1,
          }}
        >
          {phase === "submitting" ? "Recording…" : "I EXECUTED THIS"}
        </button>
        <button
          disabled={phase === "submitting"}
          onClick={() => submit("USER_IGNORED")}
          style={{
            flex: 1,
            padding: "12px 16px",
            background: "var(--bg-card)",
            border: "1px solid var(--border)",
            borderRadius: "6px",
            fontSize: "13px",
            fontWeight: "600",
            color: "var(--text-secondary)",
            cursor: phase === "submitting" ? "not-allowed" : "pointer",
            opacity: phase === "submitting" ? 0.6 : 1,
          }}
        >
          IGNORE
        </button>
      </div>
      {phase === "error" && errorMsg && (
        <p
          style={{
            fontSize: "11px",
            color: "#ef4444",
            textAlign: "center",
            marginTop: "8px",
          }}
        >
          {errorMsg}
        </p>
      )}
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