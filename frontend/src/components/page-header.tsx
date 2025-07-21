import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { ChevronDown, Calendar } from "lucide-react"

export function PageHeader() {
  return (
    <div className="flex items-center justify-between">
      <h1 className="text-3xl font-bold">Overview</h1>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 text-sm">
          <span className="text-muted-foreground">Current Node</span>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm">
                Omega <ChevronDown className="ml-1 h-3 w-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem>Omega</DropdownMenuItem>
              <DropdownMenuItem>Alpha</DropdownMenuItem>
              <DropdownMenuItem>Beta</DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              <Calendar className="mr-2 h-3 w-3" />
              24 hours <ChevronDown className="ml-1 h-3 w-3" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem>Last hour</DropdownMenuItem>
            <DropdownMenuItem>24 hours</DropdownMenuItem>
            <DropdownMenuItem>7 days</DropdownMenuItem>
            <DropdownMenuItem>30 days</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              sats <ChevronDown className="ml-1 h-3 w-3" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem>sats</DropdownMenuItem>
            <DropdownMenuItem>BTC</DropdownMenuItem>
            <DropdownMenuItem>USD</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}
