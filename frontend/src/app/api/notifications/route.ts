import { NextRequest, NextResponse } from "next/server"
import { getServerSession } from "next-auth/next"
import { authOptions } from "@/lib/auth"

interface SessionWithAccessToken {
  accessToken?: string;
  user?: {
    id: string;
    name?: string;
    email?: string;
    username?: string;
  }
}

export async function GET() {
  try {
    console.log("GET /api/notifications - retrieving notifications...");
    
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

    // Call the backend API
    const backendUrl = process.env.BACKEND_URL || "http://localhost:3030"
    
    console.log("Calling backend API:", `${backendUrl}/api/notification`);
    
    const response = await fetch(`${backendUrl}/api/notification`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${session.accessToken}`,
      },
    })

    console.log("Backend response status:", response.status);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      console.log("Backend error:", errorData);
      return NextResponse.json(
        { error: errorData.message || "Failed to retrieve notifications" },
        { status: response.status }
      )
    }

    const result = await response.json()
    console.log("Backend success:", result);
    
    return NextResponse.json(result, { status: 200 })
  } catch (error) {
    console.error("Notification GET API error:", error)
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    )
  }
}

export async function POST(request: NextRequest) {
  try {
    console.log("API route called - getting session...");
    
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
    const { name, notification_type, url } = body

    // Validate required fields
    if (!name || !notification_type || !url) {
      return NextResponse.json(
        { error: "Missing required fields: name, notification_type, and url are required" },
        { status: 400 }
      )
    }

    // Validate notification_type
    if (!["Webhook", "Discord"].includes(notification_type)) {
      return NextResponse.json(
        { error: "Invalid notification_type. Must be 'Webhook' or 'Discord'" },
        { status: 400 }
      )
    }

    // Validate URL format based on type
    if (notification_type === "Webhook" && !url.startsWith("https://")) {
      return NextResponse.json(
        { error: "Webhook URL must start with https://" },
        { status: 400 }
      )
    }

    if (notification_type === "Discord" && !url.startsWith("https://discord.com/api/webhooks/")) {
      return NextResponse.json(
        { error: "Discord URL must be a valid Discord webhook URL" },
        { status: 400 }
      )
    }

    // Call the backend API
    const backendUrl = process.env.BACKEND_URL || "http://localhost:3030"
    
    console.log("Calling backend API:", `${backendUrl}/api/notification`);
    console.log("Request data:", { name, notification_type, url });
    
    const response = await fetch(`${backendUrl}/api/notification`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${session.accessToken}`,
      },
      body: JSON.stringify({
        name,
        notification_type,
        url,
      }),
    })

    console.log("Backend response status:", response.status);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      console.log("Backend error:", errorData);
      return NextResponse.json(
        { error: errorData.message || `Failed to create ${notification_type.toLowerCase()} notification` },
        { status: response.status }
      )
    }

    const result = await response.json()
    console.log("Backend success:", result);
    
    return NextResponse.json(
      { 
        message: `${notification_type} notification created successfully!`,
        data: result 
      },
      { status: 201 }
    )
  } catch (error) {
    console.error("Notification API error:", error)
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    )
  }
} 