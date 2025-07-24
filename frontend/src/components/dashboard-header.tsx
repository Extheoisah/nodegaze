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

interface DashboardHeaderProps {
  pageTitle: string;
}

export function DashboardHeader({ pageTitle }: DashboardHeaderProps) {
  const showPageTitle = pageTitle.toLowerCase() === "events";

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
                <span className="font-medium">Omega</span>
                <span className="text-xs text-muted-foreground">
                  0x286...344
                </span>
              </div>
              <div className="h-8 w-8 rounded-full bg-green-500" />
              <ChevronDown className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem>Profile</DropdownMenuItem>
            <DropdownMenuItem>Settings</DropdownMenuItem>
            <DropdownMenuItem>Sign out</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  );
}
