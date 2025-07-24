import { withAuth } from "next-auth/middleware"

export default withAuth(
  function middleware(req) {
    // Additional middleware logic can go here
  },
  {
    callbacks: {
      authorized: ({ token }) => !!token,
    },
  }
)

export const config = {
  matcher: [
    "/overview/:path*",
    "/channels/:path*", 
    "/events/:path*",
    "/api/protected/:path*"
  ]
} 