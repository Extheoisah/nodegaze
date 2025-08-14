// app/api/channels/[id]/route.ts
import { NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth";

interface SessionWithAccessToken {
  accessToken?: string;
}

export async function GET(
  request: Request,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params; // ✅ unwrap the promise
    console.log("GET /api/channels/[id] - id:", id);

    if (!id) {
      return NextResponse.json(
        { success: false, error: "Missing channel id" },
        { status: 400 }
      );
    }

    const session = (await getServerSession(authOptions)) as SessionWithAccessToken;
    if (!session?.accessToken) {
      return NextResponse.json(
        { success: false, error: "Unauthorized - no access token" },
        { status: 401 }
      );
    }

    const backendBase = process.env.BACKEND_URL || "http://localhost:3030";
    const backendUrl = new URL(`${backendBase}/api/channels/${encodeURIComponent(id)}`);

    const incoming = new URL(request.url);
    incoming.searchParams.forEach((value, key) => {
      backendUrl.searchParams.append(key, value);
    });

    const response = await fetch(backendUrl.toString(), {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${session.accessToken}`,
      },
    });

    const result = await response.json();
    if (!response.ok) {
      return NextResponse.json(
        {
          success: false,
          error: result.error || result.message || "Failed to fetch channel details",
        },
        { status: response.status }
      );
    }

    return NextResponse.json(result);
  } catch (error) {
    console.error("Error fetching channel detail:", error);
    return NextResponse.json(
      {
        success: false,
        error: error instanceof Error ? error.message : "Internal server error",
      },
      { status: 500 }
    );
  }
}
