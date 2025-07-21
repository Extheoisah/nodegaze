import type React from "react"
import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { TrendingUp, TrendingDown } from "lucide-react"
import { cn } from "@/lib/utils"

interface MetricCardProps {
  title: string
  value: string | number
  status?: string
  statusColor?: "green" | "yellow" | "red"
  trend?: {
    value: string
    direction: "up" | "down"
  }
  chart?: React.ReactNode
}

export function MetricCard({ title, value, status, statusColor = "green", trend, chart }: MetricCardProps) {
  const statusColors = {
    green: "bg-green-100 text-green-800",
    yellow: "bg-yellow-100 text-yellow-800",
    red: "bg-red-100 text-red-800",
  }

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center gap-2">
          <div className="h-4 w-4 rounded bg-muted" />
          <span className="text-sm font-medium text-muted-foreground">{title}</span>
        </div>
        {status && <Badge className={cn("w-fit text-xs", statusColors[statusColor])}>{status}</Badge>}
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          <div className="text-2xl font-bold">{value}</div>

          {trend && (
            <div className="flex items-center gap-1">
              {trend.direction === "up" ? (
                <TrendingUp className="h-3 w-3 text-green-600" />
              ) : (
                <TrendingDown className="h-3 w-3 text-red-600" />
              )}
              <span className={cn("text-xs font-medium", trend.direction === "up" ? "text-green-600" : "text-red-600")}>
                {trend.value}
              </span>
            </div>
          )}

          {chart && <div className="h-12 w-full">{chart}</div>}
        </div>
      </CardContent>
    </Card>
  )
}
