import { cn } from "@/lib/utils"
interface MiniChartProps {
  color: "green" | "red" | "yellow"
}

export function MiniChart({ color }: MiniChartProps) {
  const colors = {
    green: "bg-green-100",
    red: "bg-red-100",
    yellow: "bg-yellow-100",
  }

  return (
    <div className={cn("h-12 w-full rounded", colors[color])}>
      <svg width="100%" height="100%" viewBox="0 0 100 48" className="overflow-visible">
        <path
          d="M0,40 Q25,20 50,30 T100,25"
          stroke={color === "green" ? "#16a34a" : color === "red" ? "#dc2626" : "#eab308"}
          strokeWidth="2"
          fill="none"
          className="opacity-60"
        />
        <path
          d="M0,40 Q25,20 50,30 T100,25 L100,48 L0,48 Z"
          fill={color === "green" ? "#16a34a" : color === "red" ? "#dc2626" : "#eab308"}
          className="opacity-10"
        />
      </svg>
    </div>
  )
}
