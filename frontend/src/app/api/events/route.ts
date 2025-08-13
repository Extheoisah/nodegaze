import { NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth";

interface SessionWithAccessToken {
  accessToken?: string;
}

export async function GET() {
  try {
    console.log("GET /api/events - retrieving events...");

    const session = (await getServerSession(authOptions)) as SessionWithAccessToken;

    if (!session?.accessToken) {
      console.log("No access token found in session");
      return NextResponse.json(
        { success: false, error: "Unauthorized - no access token" },
        { status: 401 }
      );
    }

    console.log("Session found, making request to backend...");

    const backendUrl = `${process.env.BACKEND_URL || "http://localhost:3030"}/api/events`;
    console.log("Backend URL:", backendUrl);

    const response = await fetch(backendUrl, {
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
          error: result.error || result.message || "Failed to fetch events",
        },
        { status: response.status }
      );
    }

    console.log("Events retrieved successfully");
    return NextResponse.json(result);
  } catch (error) {
    console.error("Error fetching events:", error);
    return NextResponse.json(
      {
        success: false,
        error: error instanceof Error ? error.message : "Internal server error",
      },
      { status: 500 }
    );
  }
}
