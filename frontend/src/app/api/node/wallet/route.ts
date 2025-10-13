import { NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth";

interface SessionWithAccessToken {
  accessToken?: string;
}

export async function GET() {
  try {
    console.log("GET /api/node/wallet - retrieving wallet balance...");

    const session = (await getServerSession(
      authOptions
    )) as SessionWithAccessToken;

    if (!session?.accessToken) {
      console.log("No access token found in session");
      return NextResponse.json(
        { success: false, error: "Unauthorized - no access token" },
        { status: 401 }
      );
    }

    console.log("Session found, forwarding request to backend...");

    const backendBase = process.env.BACKEND_URL || "http://localhost:3030";
    const backendUrl = `${backendBase}/api/node/wallet/balance`;

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
          error:
            result.error || result.message || "Failed to fetch wallet balance",
        },
        { status: response.status }
      );
    }

    console.log("Wallet balance retrieved successfully");
    return NextResponse.json(result);
  } catch (error) {
    console.error("Error fetching wallet balance:", error);
    return NextResponse.json(
      {
        success: false,
        error: error instanceof Error ? error.message : "Internal server error",
      },
      { status: 500 }
    );
  }
}
