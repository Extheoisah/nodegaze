import { withAuth } from "next-auth/middleware";
import { NextResponse } from "next/server";

const nodeRequiredRoutes = ["/channels", "/payments", "/nodes", "/invoices"];

export default withAuth(
  async function middleware(req) {
    const pathname = req.nextUrl.pathname;

    const requiresNodeCredentials = nodeRequiredRoutes.some((route) =>
      pathname.startsWith(route)
    );

    if (!requiresNodeCredentials) {
      return NextResponse.next();
    }

    // For routes that require node credentials, check if user has them
    try {
      const backendUrl = process.env.BACKEND_URL || "http://localhost:3030";
      const token = req.nextauth.token?.accessToken;

      if (!token) {
        return NextResponse.redirect(new URL("/login", req.url));
      }

      const response = await fetch(`${backendUrl}/api/credential/status`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const result = await response.json();

        if (!result.success || !result.data?.has_credential) {
          const url = new URL("/overview", req.url);
          url.searchParams.set(
            "message",
            "Please connect your node to access this feature"
          );
          return NextResponse.redirect(url);
        }
      }
    } catch (error) {
      console.error("Error checking credential status in middleware:", error);
    }

    return NextResponse.next();
  },
  {
    callbacks: {
      authorized: ({ token }) => !!token,
    },
  }
);

export const config = {
  matcher: [
    "/overview/:path*",
    "/channels/:path*",
    "/events/:path*",
    "/profile/:path*",
    "/payments/:path*",
    "/nodes/:path*",
    "/invoices/:path*",
  ],
};
