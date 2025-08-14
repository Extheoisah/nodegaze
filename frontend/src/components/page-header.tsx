"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ChevronDown, Calendar } from "lucide-react";
import { usePathname } from "next/navigation";

export function PageHeader() {
  const pathname = usePathname();
  const last = pathname.split("/").pop() ?? "";
  const pageTitle = last ? last.charAt(0).toUpperCase() + last.slice(1) : "";

  const [currentNode, setCurrentNode] = useState<string>("Omega");
  const [timeRange, setTimeRange] = useState<string>("Last hour");
  const [unit, setUnit] = useState<string>("sats");

    if (!pageTitle) return null;


  const nodes = ["Omega", "Alpha", "Beta"] as const;
  const ranges = ["Last hour", "24 hours", "7 days", "30 days"] as const;
  const units = ["sats", "BTC", "USD"] as const;

  return (
    <div className="flex items-center justify-between mb-2 mt-4 font-clash-grotesk text-grey-dark">
      <h1 className="text-3xl font-medium">{pageTitle}</h1>

      <div className="flex items-center gap-4">
        {/* Current Node */}
        <div className="flex items-center gap-2 text-sm">
          <span className="text-muted-foreground">Current Node</span>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm">
                {currentNode} <ChevronDown className="ml-1 h-3 w-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              {nodes.map((n) => (
                <DropdownMenuItem key={n} onSelect={() => setCurrentNode(n)}>
                  {n}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        {/* Time Range */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              <Calendar className="mr-2 h-3 w-3" />
              {timeRange} <ChevronDown className="ml-1 h-3 w-3" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            {ranges.map((r) => (
              <DropdownMenuItem key={r} onSelect={() => setTimeRange(r)}>
                {r}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        {/* Unit */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              {unit} <ChevronDown className="ml-1 h-3 w-3" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            {units.map((u) => (
              <DropdownMenuItem key={u} onSelect={() => setUnit(u)}>
                {u}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}

