"use client"

import React from "react";
import { AppLayout } from "@/components/app-layout";
import { PaymentHeader } from "@/components/payment-header";
import { DataTable } from "@/components/payment-table";
import { PaymentCard } from "@/components/payment-card";
import node from "../../../public/node.svg";
import { usePaymentMetrics, type PaymentFilters } from "@/hooks/use-payments";

const paymentTypes = [
  { label: "All Payments", key: "all" },
  { label: "Incoming Payments", key: "incoming" },
  { label: "Outgoing Payments", key: "outgoing" },
];

export default function Page() {
  const [selectedState, setSelectedState] = React.useState<string>("all");
  const [filters, setFilters] = React.useState<PaymentFilters>({});
  const [isFiltered, setIsFiltered] = React.useState<boolean>(false);

  const { data: metricsData } = usePaymentMetrics(isFiltered ? filters : undefined);
  
  const metrics = metricsData ?? {
    incomingCount: 0,
    outgoingCount: 0,
    totalIncomingAmount: 0,
    totalOutgoingAmount: 0,
    allCount: 0,
  };

  const handleApplyFilters = (applied: PaymentFilters) => {
    const hasActiveFilters = !!(
      applied.paymentState || 
      applied.operator || 
      applied.value || 
      applied.from || 
      applied.to
    );

    setFilters(applied);
    setIsFiltered(hasActiveFilters);
  };

  const formatSats = (amount: number) => {
    return new Intl.NumberFormat("en-US").format(amount);
  };

  const paymentCards = [
    {
      title: "Total Incoming Amount",
      value: `${formatSats(metrics.totalIncomingAmount)} sats`,
      statusColor: "green" as const,
      icon: node,
    },
    {
      title: "Total Outgoing Amount",
      value: `${formatSats(metrics.totalOutgoingAmount)} sats`,
      statusColor: "green" as const,
      icon: node,
    },
    {
      title: "Incoming Payments",
      value: `${formatSats(metrics.incomingCount)} payments`,
      statusColor: "green" as const,
      icon: node,
    },
    {
      title: "Outgoing Payments",
      value: `${formatSats(metrics.outgoingCount)} payments`,
      statusColor: "green" as const,
      icon: node,
    },
  ];

  return (
    <AppLayout>
      <PaymentHeader onApplyFilters={handleApplyFilters} />
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {paymentCards.map((metric, index) => (
          <PaymentCard
            key={index}
            title={metric.title}
            value={metric.value}
            statusColor={metric.statusColor}
            icon={metric.icon}
          />
        ))}
      </div>

      {/* Payment selection buttons */}
      <div className="w-[70%] text-[15px] font-[500] flex gap-[15px] my-6">
        {paymentTypes.map((type) => (
          <button
            key={type.key}
            type="button"
            onClick={() => setSelectedState(type.key)}
            className={`
              border-[1px]
              rounded-[50px]
              px-[20px]
              py-[5px]
              flex
              justify-center
              items-center
              gap-[10px]
              transition-colors
              duration-150
              ${
                selectedState === type.key
                  ? "bg-[#EFF6FF] border-blue-500 text-[#204ECF]"
                  : "bg-[#ededed] border-transparent text-[#344054] hover:bg-[#e0e7ef]"
              }
            `}
          >
            <p>{type.label}</p>
            <div 
            className={`
              border-[1px]
              rounded-[50px]
              px-[15px]
              py-[5px]
              flex
              justify-center
              items-center
              gap-[10px]
              transition-colors
              duration-150
              ${
                selectedState === type.key
                  ? "bg-[#204ECF]  border-blue-500 text-[#FFFFFF]"
                  : "bg-[#ededed] border-transparent text-[#344054] hover:bg-[#e0e7ef]"
              }
            `}
            >{type.key === "incoming" ? metrics.incomingCount : type.key === "all" ? metrics.allCount : type.key === "outgoing" ? metrics.outgoingCount : 0}</div>
          </button>
        ))}
      </div>

      <div className="h-full">
        <DataTable selectedState={selectedState} filters={filters} />
      </div>
    </AppLayout>
  );
}