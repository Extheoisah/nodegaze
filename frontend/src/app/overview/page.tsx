import { AppLayout } from "@/components/app-layout"
import { PageHeader } from "@/components/page-header"
import { MetricCard } from "@/components/metric-card"
import { MiniChart } from "@/components/mini-chart"

const metricsData = [
  {
    title: "Node Performance",
    value: "97",
    status: "Very Good",
    statusColor: "green" as const,
    chart: <MiniChart color="green" />,
  },
  {
    title: "Node Availability",
    value: "54",
    status: "Good",
    statusColor: "yellow" as const,
    chart: <MiniChart color="yellow" />,
  },
  {
    title: "Channel Count",
    value: "5",
    trend: { value: "7.2%", direction: "down" as const },
    chart: <MiniChart color="red" />,
  },
  {
    title: "Node Liquidity",
    value: "100,000,000 sats",
    trend: { value: "7.2%", direction: "up" as const },
    chart: <MiniChart color="green" />,
  },
  {
    title: "Inbound Balance",
    value: "500,000,000 sats",
    trend: { value: "7.2%", direction: "up" as const },
    chart: <MiniChart color="green" />,
  },
  {
    title: "Outbound Balance",
    value: "150,000,000 sats",
    trend: { value: "7.2%", direction: "down" as const },
    chart: <MiniChart color="red" />,
  },
  {
    title: "Onchain balance",
    value: "100,000,000 sats",
    trend: { value: "7.2%", direction: "up" as const },
    chart: <MiniChart color="green" />,
  },
  {
    title: "Offchain balance",
    value: "100,000,000 sats",
    trend: { value: "7.2%", direction: "up" as const },
    chart: <MiniChart color="green" />,
  },
]

export default function Dashboard() {
  return (
    <AppLayout>
      <PageHeader />

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {metricsData.map((metric, index) => (
          <MetricCard
            key={index}
            title={metric.title}
            value={metric.value}
            status={metric.status}
            statusColor={metric.statusColor}
            trend={metric.trend}
            chart={metric.chart}
          />
        ))}
      </div>
    </AppLayout>
  )
}
