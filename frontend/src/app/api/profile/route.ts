// app/api/channels/route.ts
import { NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth";

interface SessionWithAccessToken {
  accessToken?: string;
}

export async function GET(request: Request) {
  try {
    console.log("GET /api/profile - retrieving user details...");

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
    const id = incomingUrl.searchParams.get("id");
    if (!id) {
      return NextResponse.json(
        { success: false, error: "Missing required user id" },
        { status: 400 }
      );
    }
    const backendUrl = new URL(`${backendBase}/api/user/get-user/${encodeURIComponent(id)}`);

    // forward all query params from the incoming request
    incomingUrl.searchParams.forEach((value, key) => {
      backendUrl.searchParams.append(key, value);
    });

    // no pagination for single user fetch

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
          error: result.error || result.message || "Failed to fetch User details",
        },
        { status: response.status }
      );
    }

    console.log("Channels retrieved successfully");
    return NextResponse.json(result);
  } catch (error) {
    console.error("Error fetching channels:", error);
    return NextResponse.json(
      {
        success: false,
        error: error instanceof Error ? error.message : "Internal server error",
      },
      { status: 500 }
    );
  }
}
