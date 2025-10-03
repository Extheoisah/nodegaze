import { NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth";

interface SessionWithAccessToken {
  accessToken?: string;
}

export async function GET(request: Request) {
  try {
    const session = (await getServerSession(authOptions)) as SessionWithAccessToken;
    if (!session?.accessToken) {
      return NextResponse.json(
        { success: false, error: "Unauthorized - no access token" },
        { status: 401 }
      );
    }

    const incomingUrl = new URL(request.url);
    const backendBase = process.env.BACKEND_URL || "http://localhost:3030";
    const backendUrl = new URL(`${backendBase}/api/account/get-account-users`);

    // Forward pagination or other query params transparently
    incomingUrl.searchParams.forEach((value, key) => {
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
          error: result.error || result.message || "Failed to fetch account users",
        },
        { status: response.status }
      );
    }

    return NextResponse.json(result);
  } catch (error) {
    return NextResponse.json(
      {
        success: false,
        error: error instanceof Error ? error.message : "Internal server error",
      },
      { status: 500 }
    );
  }
}


