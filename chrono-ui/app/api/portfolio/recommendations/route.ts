/**
 * Next.js API route: POST /api/portfolio/recommendations
 *
 * Thin proxy to the Rust backend at POST /api/v0/portfolio/recommendations.
 * Keeps the frontend decoupled from the backend host/port.
 *
 * v0.2: client sends only UserProfile + PortfolioContext.
 * The backend fetches certified decisions from its own intelligence source.
 */

import { NextRequest, NextResponse } from "next/server";

const BACKEND_URL =
  process.env.CHRONOSENTIMENT_API_URL ?? "http://localhost:3000";

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();

    const upstream = await fetch(
      `${BACKEND_URL}/api/v0/portfolio/recommendations`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }
    );

    // Read the raw text first so we can handle non-JSON responses gracefully.
    const text = await upstream.text();

    let data: unknown;
    try {
      data = JSON.parse(text);
    } catch {
      // Backend returned non-JSON (HTML error page, plain text, etc.)
      return NextResponse.json(
        {
          error: `Backend returned a non-JSON response (status ${upstream.status}). ` +
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
