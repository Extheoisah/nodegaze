import { NextRequest, NextResponse } from "next/server"
import { getServerSession } from "next-auth"
import { authOptions } from "@/lib/auth"

interface SessionWithAccessToken {
  accessToken?: string;
}

export async function POST(request: NextRequest) {
  try {
    console.log("Invite API route called - getting session...");
    
    // Get the session on the server side
    const session = await getServerSession(authOptions) as SessionWithAccessToken
    
    console.log("Session retrieved:", {
      hasSession: !!session,
      hasAccessToken: !!session?.accessToken,
      sessionKeys: session ? Object.keys(session) : [],
    });
    
    if (!session?.accessToken) {
      console.log("No access token found, returning 401");
      return NextResponse.json(
        { error: "Unauthorized. Please log in again." },
        { status: 401 }
      )
    }
    
    console.log("Session valid, proceeding with request...");

    const body = await request.json()
    const { email } = body

    // Validate required fields
    if (!email) {
      return NextResponse.json(
        { error: "Email is required" },
        { status: 400 }
      )
    }

    // Validate email format
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
      return NextResponse.json(
        { error: "Invalid email format" },
        { status: 400 }
      )
    }

    // Call the backend API
    const backendUrl = process.env.BACKEND_URL || "http://localhost:3030"
    
    console.log("Calling backend API:", `${backendUrl}/api/invite/send-invite`);
    console.log("Request data:", { email });
    
    const response = await fetch(`${backendUrl}/api/invite/send-invite`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${session.accessToken}`,
      },
      body: JSON.stringify({
        email,
      }),
    })

    console.log("Backend response status:", response.status);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      console.log("Backend error:", errorData);
      return NextResponse.json(
        { error: errorData.message || "Failed to send invite" },
        { status: response.status }
      )
    }

    const result = await response.json()
    console.log("Backend success:", result);
    
    return NextResponse.json(
      { 
        message: "Invite sent successfully!",
        data: result 
      },
      { status: 200 }
    )
  } catch (error) {
    console.error("Invite API error:", error)
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    )
  }
}
