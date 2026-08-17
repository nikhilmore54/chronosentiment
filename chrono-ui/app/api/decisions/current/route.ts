/**
 * Next.js API route: GET /api/decisions/current
 *
 * Thin proxy to the Rust backend at GET /api/v0/decisions/current.
 * Returns certified C3-002 + Coralys v0 decisions including entry_price,
 * target_price, and risk_boundary (stoploss) for each instrument.
 *
 * Response shape (from backend):
 * {
 *   decisions: [{
 *     instrument, c3_002_direction,
 *     entry_price, target_pct, target_price,
 *     risk_pct, risk_boundary,          ← stoploss
 *     maximum_hold_sessions,
 *     decision_rationale, decision_id, execution_intent_id
 *   }, ...],
 *   certified_at, c3_002_artifact, coralys_artifact, universe
 * }
 */

import { NextResponse } from "next/server";

const BACKEND_URL =
  process.env.CHRONOSENTIMENT_API_URL ?? "http://localhost:3000";

export async function GET() {
  try {
    const upstream = await fetch(`${BACKEND_URL}/api/v0/decisions/current`, {
      method: "GET",
      headers: { "Content-Type": "application/json" },
      // Disable Next.js fetch cache so we always get fresh certified decisions.
      cache: "no-store",
    });

    const text = await upstream.text();

    let data: unknown;
    try {
      data = JSON.parse(text);
    } catch {
      return NextResponse.json(
        {
          error:
            `Backend returned a non-JSON response (status ${upstream.status}). ` +
            `Is the Rust backend running at ${BACKEND_URL}? ` +
            `Response preview: ${text.slice(0, 120)}`,
        },
        { status: 502 }
      );
    }

    if (!upstream.ok) {
      return NextResponse.json(data, { status: upstream.status });
    }

    return NextResponse.json(data);
  } catch (err) {
    return NextResponse.json(
      {
        error:
          `Cannot reach backend at ${BACKEND_URL}. ` +
          `Start the Rust server with: cargo run --bin chronosentiment_server. ` +
          `Detail: ${String(err)}`,
      },
      { status: 502 }
    );
  }
}