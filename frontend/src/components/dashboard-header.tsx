"use client";

import { Search, Bell, ChevronDown } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Badge } from "@/components/ui/badge";
import { useSession, signOut } from "next-auth/react";
import { useRouter } from "next/navigation";
import { useState } from "react";
import Link from "next/link";

interface DashboardHeaderProps {
  pageTitle: string;
}

export function DashboardHeader({ pageTitle }: DashboardHeaderProps) {
  const { data: session } = useSession();
  const router = useRouter();
  const [isSigningOut, setIsSigningOut] = useState(false);
  const showPageTitle = pageTitle.toLowerCase() === "events";

  const handleSignOut = async () => {
    if (isSigningOut) return; // Prevent multiple clicks

    setIsSigningOut(true);
    try {
      await signOut({
        redirect: false,
      });
      router.push("/login");
    } catch (error) {
      console.error("Sign out error:", error);
    } finally {
      setIsSigningOut(false);
    }
  };

  return (
    <header className="flex h-16 items-center justify-between mt-5 bg-background px-6">
      <div className="flex items-center gap-4">
        {showPageTitle ? (
          <h1 className="text-3xl font-bold text-grey-dark">{pageTitle}</h1>
        ) : null}
      </div>

      <div className="flex items-center gap-4">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="flex items-center gap-2">
              <div className="flex flex-col items-end text-sm">
                <span className="font-medium">
                  {session?.user?.name || session?.user?.username || "User"}
                </span>
                <span className="text-xs text-muted-foreground">
                  {session?.user?.email || "user@example.com"}
                </span>
              </div>
              <div className="h-8 w-8 rounded-full bg-green-500 flex items-center justify-center text-white text-sm font-medium">
                {(session?.user?.name || session?.user?.username || "U")
                  .charAt(0)
                  .toUpperCase()}
              </div>
              <ChevronDown className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem asChild>
              <Link href="/profile" className="cursor-pointer">
                Profile
              </Link>
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={handleSignOut}
              className="cursor-pointer"
              disabled={isSigningOut}
            >
              {isSigningOut ? "Signing out..." : "Sign out"}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  );
}
