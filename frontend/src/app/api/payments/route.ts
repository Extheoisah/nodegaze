// app/api/payments/route.ts
import { NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth";

interface SessionWithAccessToken {
  accessToken?: string;
}

export async function GET(request: Request) {
  try {
    console.log("GET /api/payments - retrieving payments...");

    const session = (await getServerSession(authOptions)) as SessionWithAccessToken;

    if (!session?.accessToken) {
      console.log("No access token found in session");
      return NextResponse.json(
        { success: false, error: "Unauthorized - no access token" },
        { status: 401 }
      );
    }

    console.log("Session found, forwarding request to backend...");

    // Build backend URL and forward incoming query params (page, per_page, etc.)
    const incomingUrl = new URL(request.url);
    const backendBase = process.env.BACKEND_URL || "http://localhost:3030";
    const backendUrl = new URL(`${backendBase}/api/payments`);

    // forward all query params from the incoming request
    incomingUrl.searchParams.forEach((value, key) => {
      backendUrl.searchParams.append(key, value);
    });

    // sensible defaults if none provided
    if (!backendUrl.searchParams.has("page")) backendUrl.searchParams.set("page", "1");
    if (!backendUrl.searchParams.has("per_page")) backendUrl.searchParams.set("per_page", "10");

    console.log("Backend URL:", backendUrl.toString());

    const response = await fetch(backendUrl.toString(), {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${session.accessToken}`,
      },
    });

    console.log("Backend response status:", response.status);

    const result = await response.json();
    console.log("Backend response:", result);

    if (!response.ok) {
      console.error("Backend error:", result);
      return NextResponse.json(
        {
          success: false,
          error: result.error || result.message || "Failed to fetch payments",
        },
        { status: response.status }
      );
    }

    console.log("Payments retrieved successfully");
    return NextResponse.json(result);
  } catch (error) {
    console.error("Error fetching payments:", error);
    return NextResponse.json(
      {
        success: false,
        error: error instanceof Error ? error.message : "Internal server error",
      },
      { status: 500 }
    );
  }
}
