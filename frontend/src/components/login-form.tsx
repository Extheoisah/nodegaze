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

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
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
          <form>
            <div className="flex flex-col gap-6">
              <div className="grid gap-3">
                <Label htmlFor="username">Username</Label>
                <Input
                  id="username"
                  placeholder="username"
                  required
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
                <Input id="password" type="password" placeholder="password" required />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="node-public-key">Node Public Key</Label>
                </div>
                <Input id="node-public-key" type="text" placeholder="026c62282d38ea38daa437041b38e696f245749820343f60800c898274e8189467" required />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="node-address">Node Address</Label>
                </div>
                <Input id="node-address" type="text" placeholder="https://192.168.122.92:10001" required />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="macaroon-path">Macaroon Path</Label>
                </div>
                <Input id="macaroon-path" type="text" placeholder="/home/user/.lnd/data/chain/bitcoin/mainnet/admin.macaroon" required />
              </div>
              <div className="grid gap-3">
                <div className="flex items-center">
                  <Label htmlFor="tls-cert-path">TLS Certificate Path</Label>
                </div>
                <Input id="tls-cert-path" type="text" placeholder="/home/user/.lnd/tls.cert" required />
              </div>
              <div className="flex flex-col gap-3">
                <Button type="submit" className="w-full cursor-pointer bg-grey-accent hover:bg-blue-primary">
                  Login
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
