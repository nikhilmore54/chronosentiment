/**
 * Next.js API route: GET /api/recommendations/v1/latest
 *
 * Proxies to the Coralys Decision Server v1 recommendations endpoint.
 * Uses RecommendationEngineV1 — ticker-specific analogue population,
 * adaptive geometry from MFE/MAE percentiles, first-exit semantics.
 *
 * Response includes: adaptive_target, adaptive_risk, adaptive_rr,
 * adaptive_horizon_sessions, degradation_level, target_rate, sample_size,
 * vol_regime, volume_regime, policy_version="v1".
 */
import { NextResponse } from "next/server";

const CORALYS_API = process.env.CORALYS_API_URL ?? "http://localhost:3001";

export async function GET() {
  try {
    const res = await fetch(`${CORALYS_API}/recommendations/v1/latest`, {
      cache: "no-store",
    });
    if (!res.ok) {
      return NextResponse.json(
        { error: `Decision Server returned ${res.status}` },
        { status: res.status }
      );
    }
    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json(
      { error: "Decision Server unavailable" },
      { status: 503 }
    );
  }
}