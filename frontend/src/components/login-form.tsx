"use client"

import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { useState } from "react"
import { signIn } from "next-auth/react"
import { useRouter } from "next/navigation"

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState("")
  const router = useRouter()

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    setIsLoading(true)
    setError("")

    const formData = new FormData(e.currentTarget)
    const credentials = {
      username: formData.get("username") as string,
      password: formData.get("password") as string,
      nodePublicKey: formData.get("nodePublicKey") as string,
      nodeAddress: formData.get("nodeAddress") as string,
      macaroonPath: formData.get("macaroonPath") as string,
      tlsCertPath: formData.get("tlsCertPath") as string,
    }

    try {
      const result = await signIn("credentials", {
        ...credentials,
        redirect: false,
      })

      if (result?.error) {
        setError("Invalid credentials or connection failed")
        return
      }

      if (result?.ok) {
        router.push("/overview")
      }
    } catch (error) {
      setError("An unexpected error occurred")
      console.error("Login error:", error)
    } finally {
      setIsLoading(false)
    }
  }
  return (
    <div className={cn("flex flex-col gap-6 font-clash-grotesk", className)} {...props}>
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="text-2xl font-semibold text-grey-dark">Login to your account</CardTitle>
          <CardDescription>
            Enter your credentials below to login to your account
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit}>
            <div className="flex flex-col gap-6">
              {error && (
                <div className="p-3 text-sm text-red-600 bg-red-50 rounded-md border border-red-200">
                  {error}
                </div>
              )}
              <div className="grid gap-3">
                <Label htmlFor="username">Username</Label>
                <Input
                  id="username"
                  name="username"
                  placeholder="username"
                  required
                  disabled={isLoading}
                />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="password">Password</Label>
                  {/* <a
                    href="#"
                    className="ml-auto inline-block text-sm underline-offset-4 hover:underline"
                  >
                    Forgot your password?
                  </a> */}
                </div>
                <Input 
                  id="password" 
                  name="password"
                  type="password" 
                  placeholder="password" 
                  required 
                  disabled={isLoading}
                />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="node-public-key">Node Public Key</Label>
                </div>
                <Input 
                  id="node-public-key" 
                  name="nodePublicKey"
                  type="text" 
                  placeholder="026c62282d38ea38daa437041b38e696f245749820343f60800c898274e8189467" 
                  required 
                  disabled={isLoading}
                />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="node-address">Node Address</Label>
                </div>
                <Input 
                  id="node-address" 
                  name="nodeAddress"
                  type="text" 
                  placeholder="https://192.168.122.92:10001" 
                  required 
                  disabled={isLoading}
                />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="macaroon-path">Macaroon Path</Label>
                </div>
                <Input 
                  id="macaroon-path" 
                  name="macaroonPath"
                  type="text" 
                  placeholder="/home/user/.lnd/data/chain/bitcoin/mainnet/admin.macaroon" 
                  required 
                  disabled={isLoading}
                />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="tls-cert-path">TLS Certificate Path</Label>
                </div>
                <Input 
                  id="tls-cert-path" 
                  name="tlsCertPath"
                  type="text" 
                  placeholder="/home/user/.lnd/tls.cert" 
                  required 
                  disabled={isLoading}
                />
              </div>
              <div className="flex flex-col gap-3">
                <Button 
                  type="submit" 
                  className="w-full cursor-pointer bg-grey-accent hover:bg-blue-primary disabled:opacity-50"
                  disabled={isLoading}
                >
                  {isLoading ? "Logging in..." : "Login"}
                </Button>
              </div>
            </div>
            <div className="mt-4 text-center text-sm">
              Don&apos;t have an account?{" "}
              <a href="/signup" className="underline underline-offset-4">
                Sign up
              </a>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
