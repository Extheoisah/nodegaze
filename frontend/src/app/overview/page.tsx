"use client"

import { AppLayout } from "@/components/app-layout"
import { PageHeader } from "@/components/page-header"
import { MetricCard } from "@/components/metric-card"
import { ConnectNodeDialog } from "@/components/connect-node-dialog"
import { useEffect, useState } from "react"
import { useSearchParams } from "next/navigation"
import { useNodeOverview } from "@/hooks/use-node-overview"
import node from "../../../public/node.svg"

export default function Dashboard() {
  const searchParams = useSearchParams()
  const [hasCredential, setHasCredential] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [successMessage, setSuccessMessage] = useState("")
  const [redirectMessage, setRedirectMessage] = useState("")

  const { metrics, isLoading: metricsLoading } = useNodeOverview()

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

  const formatSats = (amount: number) => {
    return new Intl.NumberFormat("en-US").format(amount);
  };

  const metricsData = metrics ? [
    {
      title: "Channel Count",
      value: String(metrics.channelCount),
      icon: node,
    },
    {
      title: "Active Channels",
      value: String(metrics.activeChannelCount),
      icon: node,
    },
    {
      title: "Total Capacity",
      value: `${formatSats(metrics.totalCapacitySat)} sats`,
      icon: node,
    },
    {
      title: "Inbound Balance",
      value: `${formatSats(metrics.totalRemoteBalanceSat)} sats`,
      icon: node,
    },
    {
      title: "Outbound Balance",
      value: `${formatSats(metrics.totalLocalBalanceSat)} sats`,
      icon: node,
    },
    {
      title: "Onchain Balance",
      value: `${formatSats(metrics.onchainBalanceSat)} sats`,
      icon: node,
    },
    {
      title: "Total Payments",
      value: String(metrics.totalPayments),
      icon: node,
    },
    {
      title: "Settled Payments",
      value: String(metrics.settledPayments),
      icon: node,
    },
    {
      title: "Total Incoming Volume",
      value: `${formatSats(metrics.totalIncomingVolumeSat)} sats`,
      icon: node,
    },
    {
      title: "Total Outgoing Volume",
      value: `${formatSats(metrics.totalOutgoingVolumeSat)} sats`,
      icon: node,
    },
  ] : [];

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

      {metricsLoading && hasCredential && (
        <div className="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-blue-800 font-medium">Loading node metrics...</p>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {metricsData.map((metric, index) => (
          <MetricCard
            key={index}
            title={metric.title}
            value={metric.value}
            icon={metric.icon}
          />
        ))}
      </div>
    </AppLayout>
  )
}
