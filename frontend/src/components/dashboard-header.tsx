"use client"

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
        ) : (
          <div className="relative w-96">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Search"
              className="pl-10 border-1 border-[oklch(0.8715 0.0123 259.82)] h-11 rounded-md bg-muted/0"
            />
          </div>
        )}
      </div>

      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="h-4 w-4" />
          <Badge className="absolute -top-1 -right-1 h-5 w-5 rounded-full p-0 text-xs">
            1
          </Badge>
        </Button>

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
                {(session?.user?.name || session?.user?.username || "U").charAt(0).toUpperCase()}
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
            <DropdownMenuItem>Settings</DropdownMenuItem>
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
