"use client"

import { SignupForm } from "@/components/signup-form"
import { useSession } from "next-auth/react"
import { useRouter } from "next/navigation"
import { useEffect } from "react"

export default function Page() {
  const { data: session, status } = useSession()
  const router = useRouter()

  useEffect(() => {
    // If user is authenticated, redirect to overview
    if (status === "authenticated" && session) {
      router.push("/overview")
    }
  }, [session, status, router])

  // Show loading while checking authentication
  if (status === "loading") {
    return (
      <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
        <div className="w-full max-w-lg">
          <div className="flex items-center justify-center">
            <div className="text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900 mx-auto"></div>
              <p className="mt-4 text-sm text-muted-foreground">Checking authentication...</p>
            </div>
          </div>
        </div>
      </div>
    )
  }

  // Only show signup form if user is not authenticated
  if (status === "unauthenticated") {
    return (
      <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
        <div className="w-full max-w-lg">
          <SignupForm />
        </div>
      </div>
    )
  }

  // Return null if redirecting (authenticated users)
  return null
} 