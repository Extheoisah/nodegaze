"use client"

import { AppLayout } from "@/components/app-layout"
import { PageHeader } from "@/components/page-header"
import { MetricCard } from "@/components/metric-card"
import { MiniChart } from "@/components/mini-chart"
import { ConnectNodeDialog } from "@/components/connect-node-dialog"
import { useEffect, useState } from "react"
import { useSearchParams } from "next/navigation"
import node from "../../../public/node.svg"

const metricsData = [
  {
    title: "Node Performance",
    value: "97",
    status: "Very Good",
    statusColor: "green" as const,
    chart: <MiniChart color="green" />,
    icon: node,
  },
  {
    title: "Node Availability",
    value: "54",
    status: "Good",
    statusColor: "yellow" as const,
    chart: <MiniChart color="yellow" />,
     icon: node,
  },
  {
    title: "Channel Count",
    value: "5",
    trend: { value: "7.2%", direction: "down" as const },
    chart: <MiniChart color="red" />,
     icon: node,
  },
  {
    title: "Node Liquidity",
    value: "100,000,000 sats",
    trend: { value: "7.2%", direction: "up" as const },
    chart: <MiniChart color="green" />,
     icon: node,
  },
  {
    title: "Inbound Balance",
    value: "500,000,000 sats",
    trend: { value: "7.2%", direction: "up" as const },
    chart: <MiniChart color="green" />,
     icon: node,
  },
  {
    title: "Outbound Balance",
    value: "150,000,000 sats",
    trend: { value: "7.2%", direction: "down" as const },
    chart: <MiniChart color="red" />,
     icon: node,
  },
  {
    title: "Onchain balance",
    value: "100,000,000 sats",
    trend: { value: "7.2%", direction: "up" as const },
    chart: <MiniChart color="green" />,
     icon: node,
  },
  {
    title: "Offchain balance",
    value: "100,000,000 sats",
    trend: { value: "7.2%", direction: "up" as const },
    chart: <MiniChart color="green" />,
     icon: node,
  },
]

export default function Dashboard() {
  const searchParams = useSearchParams()
  const [hasCredential, setHasCredential] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [successMessage, setSuccessMessage] = useState("")
  const [redirectMessage, setRedirectMessage] = useState("")

  const checkCredentialStatus = async () => {
    try {
      const response = await fetch("/api/credential/status")
      if (response.ok) {
        const result = await response.json()
        if (result.success && result.data) {
          setHasCredential(result.data.has_credential)
        }
      }
    } catch (error) {
      console.error("Error checking credential status:", error)
    } finally {
      setIsLoading(false)
    }
  }

  useEffect(() => {
    checkCredentialStatus()
    
    // Check for redirect message
    const message = searchParams.get("message")
    if (message) {
      setRedirectMessage(message)
      setTimeout(() => setRedirectMessage(""), 8000)
    }
  }, [searchParams])

  const handleConnectSuccess = () => {
    setHasCredential(true)
    setSuccessMessage("Node connected successfully!")
    setTimeout(() => setSuccessMessage(""), 5000)
  }

  return (
    <AppLayout>
      <PageHeader />

      {redirectMessage && (
        <div className="mb-6 p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
          <p className="text-yellow-800 font-medium">{redirectMessage}</p>
        </div>
      )}

      {!isLoading && !hasCredential && (
        <div className="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-lg flex items-center justify-between">
          <div>
            <h3 className="font-semibold text-blue-900">Connect Your Lightning Node</h3>
            <p className="text-sm text-blue-700">
              Connect your Lightning node to start monitoring your channels, payments, and invoices.
            </p>
          </div>
          <ConnectNodeDialog onSuccess={handleConnectSuccess} />
        </div>
      )}

      {successMessage && (
        <div className="mb-6 p-4 bg-green-50 border border-green-200 rounded-lg">
          <p className="text-green-800 font-medium">{successMessage}</p>
        </div>
      )}

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
            icon={metric.icon}
          />
        ))}
      </div>
    </AppLayout>
  )
}
