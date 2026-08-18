/**
 * Next.js API route: GET /api/recommendations/latest
 *
 * Proxies to the Coralys Decision Server so the browser-side /live page
 * avoids cross-origin restrictions. The server-side fetch runs in the
 * Next.js Node.js process, which has no CORS constraint.
 */
import { NextResponse } from "next/server";

const CORALYS_API = process.env.CORALYS_API_URL ?? "http://localhost:3001";

export async function GET() {
  try {
    const res = await fetch(`${CORALYS_API}/recommendations/latest`, {
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