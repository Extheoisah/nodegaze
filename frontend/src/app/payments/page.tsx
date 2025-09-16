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
  { label: "Outgoing Payments", key: "outgoing" },
  { label: "Incoming Payments", key: "incoming" },
  { label: "Forwarded Payments", key: "forwarded" },

];

export default function Page() {

    const [selectedType, setSelectedType] = React.useState<string>("all");
const [payments, setPayments] = React.useState<Payment[]>([]);

  
  return (
    <AppLayout>
      <PaymentHeader />
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
            onClick={() => setSelectedType(type.key)}
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
                selectedType === type.key
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
                selectedType === type.key
                  ? "bg-[#204ECF]  border-blue-500 text-[#FFFFFF]"
                  : "bg-[#ededed] border-transparent text-[#344054] hover:bg-[#e0e7ef]"
              }
            `}
            >{type.key === "all" ? payments.length : 0}</div>
          </button>
        ))}
      </div>

      <div className="h-full">
        <DataTable payments={payments} setPayments={setPayments} />
      </div>
    </AppLayout>
  );
}
