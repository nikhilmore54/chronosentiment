/**
 * Next.js API proxy — POST /api/decisions/[id]/execution
 *
 * Forwards the execution recording request to the Coralys Decision Server
 * at CORALYS_API_URL (default: http://localhost:3001).
 *
 * The client component (ExecutionRecorder) cannot call the backend directly
 * from the browser due to CORS. This proxy runs server-side.
 */

import { NextRequest, NextResponse } from "next/server";

function apiBase(): string {
  return process.env.CORALYS_API_URL ?? "http://localhost:3001";
}

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  const { id } = await params;
  const body = await request.json();

  try {
    const upstream = await fetch(
      `${apiBase()}/decisions/${encodeURIComponent(id)}/execution`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }
    );

    const data = await upstream.json();
    return NextResponse.json(data, { status: upstream.status });
  } catch (e) {
    return NextResponse.json(
      { error: e instanceof Error ? e.message : "upstream error" },
      { status: 502 }
    );
  }
}