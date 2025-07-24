"use client"

import { useSession } from "next-auth/react"
import { useRouter } from "next/navigation"
import { useEffect } from "react"

export default function Home() {
  const { data: session, status } = useSession()
  const router = useRouter()

  useEffect(() => {
    if (status === "authenticated" && session) {
      // Redirect authenticated users to overview
      router.push("/overview")
    } else if (status === "unauthenticated") {
      // Redirect unauthenticated users to login
      router.push("/login")
    }
  }, [session, status, router])

  // Show loading while checking authentication
  if (status === "loading") {
    return (
      <div className="flex min-h-screen w-full items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-900 mx-auto"></div>
          <p className="mt-6 text-lg text-muted-foreground">Loading NodeGaze...</p>
        </div>
      </div>
    )
  }

  // Return null while redirecting
  return null
}
