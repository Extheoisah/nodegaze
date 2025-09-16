"use client"


import React from "react";
import { AppLayout } from "@/components/app-layout";
import { PaymentHeader } from "@/components/payment-header";
import { DataTable } from "@/components/payment-table";
import { PaymentCard } from "@/components/payment-card";
import { MiniChart } from "@/components/mini-chart";
import node from "../../../public/node.svg";
import type { Payment } from "@/components/payment-table";



const paymentData = [
  {
    title: "Outgoing Payments(Amount)",
    trend: { value: "7.2%", direction: "up" as const },
    value: "500,000,000 sats",
    statusColor: "green" as const,
    chart: <MiniChart color="green" />,
    icon: node,
  },
  {
    title: "Outgoing Payments(Amount)",
    trend: { value: "7.2%", direction: "up" as const },
    value: "10,000 sats",
    statusColor: "green" as const,
    chart: <MiniChart color="green" />,
    icon: node,
  },
  {
    title: "Outgoing Payments(Amount)",
    trend: { value: "3.2%", direction: "down" as const },
    value: "150,000,000 sats",
    statusColor: "green" as const,
    chart: <MiniChart color="red" />,
    icon: node,
  },
  {
    title: "Outgoing Payments(Amount)",
    trend: { value: "3.2%", direction: "down" as const },
    value: "17,000 sats",
    statusColor: "green" as const,
    chart: <MiniChart color="red" />,
    icon: node,
  },

  {
    title: "Outgoing Payments(Amount)",
    trend: { value: "3.2%", direction: "down" as const },
    value: "150,000,000 sats",
    statusColor: "green" as const,
    chart: <MiniChart color="red" />,
    icon: node,
  },

  {
    title: "Outgoing Payments(Amount)",
    trend: { value: "3.2%", direction: "down" as const },
    value: "5500 sats",
    statusColor: "green" as const,
    chart: <MiniChart color="red" />,
    icon: node,
  },
];


const paymentTypes = [
  { label: "All Payments", key: "all" },
  { label: "Incoming Payments", key: "incoming" },
  { label: "Outgoing Payments", key: "outgoing" },
];

export type PaymentFilters = {
  paymentState?: "settled" | "failed" | "pending";
  operator?: "gte" | "lte" | "eq";
  value?: number;
  from?: string; // YYYY-MM-DD
  to?: string;   // YYYY-MM-DD
};

export default function Page() {

    const [selectedState, setSelectedState] = React.useState<string>("all");
const [payments, setPayments] = React.useState<Payment[]>([]);
const [incomingCount, setIncomingCount] = React.useState<number>(0);
const [outgoingCount, setOutgoingCount] = React.useState<number>(0);
const [allCount, setAllCount] = React.useState<number>(0);
const [filters, setFilters] = React.useState<PaymentFilters>({});

React.useEffect(() => {
  async function fetchIncomingCount() {
    try {
      const res = await fetch(`/api/payments?payment_types=incoming&per_page=1&page=1`);
      const data = await res.json();
      const count = data?.pagination?.total_items ?? data?.data?.items?.length ?? 0;
      setIncomingCount(Number(count) || 0);
    } catch {
      setIncomingCount(0);
    }
  }
  fetchIncomingCount();
}, []);

React.useEffect(() => {
  async function fetchAllCount() {
    try {
      const res = await fetch(`/api/payments?per_page=1&page=1`);
      const data = await res.json();
      const count = data?.pagination?.total_items ?? data?.data?.items?.length ?? 0;
      setAllCount(Number(count) || 0);
    } catch {
      setAllCount(0);
    }
  }
  fetchAllCount();
}, []);

React.useEffect(() => {
  async function fetchOutgoingCount() {
    try {
      const res = await fetch(`/api/payments?payment_types=outgoing&per_page=1&page=1`);
      const data = await res.json();
      const count = data?.pagination?.total_items ?? data?.data?.items?.length ?? 0;
      setOutgoingCount(Number(count) || 0);
    } catch {
      setOutgoingCount(0);
    }
  }
  fetchOutgoingCount();
}, []);

const handleApplyFilters = (applied: PaymentFilters) => {
  setFilters(applied);
  if (applied.paymentState) {
    setSelectedState(applied.paymentState);
  }
};
  
  return (
    <AppLayout>
      <PaymentHeader onApplyFilters={handleApplyFilters} />
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {paymentData.map((metric, index) => (
          <PaymentCard
            key={index}
            title={metric.title}
            value={metric.value}
            statusColor={metric.statusColor}
            trend={metric.trend}
            chart={metric.chart}
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
            onClick={() => {/* keeping counts clickable but not affecting selectedState */}}
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
                // keep neutral styling since not tied to selectedState now
                "bg-[#ededed] border-transparent text-[#344054] hover:bg-[#e0e7ef]"
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
                "bg-[#ededed] border-transparent text-[#344054]"
              }
            `}
            >{type.key === "incoming" ? incomingCount : type.key === "all" ? allCount : type.key === "outgoing" ? outgoingCount : 0}</div>
          </button>
        ))}
      </div>

      <div className="h-full">
        <DataTable payments={payments} setPayments={setPayments} selectedState={selectedState} filters={filters} />
      </div>
    </AppLayout>
  );
}
