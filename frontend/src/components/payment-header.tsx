"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ChevronDown } from "lucide-react";
import { usePathname } from "next/navigation";
import Image from "next/image";
import ExportData from "../../public/exportdata.svg";
import Filter from "../../public/filter.svg";
import Close from "../../public/close.svg"
import Add from "../../public/add.svg"
import CapacityIcon from "../../public/capacity-icon.svg"

export type HeaderFilters = {
  paymentType?: "all" | "incoming" | "outgoing";
  operator?: "gte" | "lte" | "eq";
  value?: number;
  from?: string;
  to?: string;
};

export function PaymentHeader({ onApplyFilters }: { onApplyFilters?: (filters: HeaderFilters) => void }) {
  const pathname = usePathname();
  const last = pathname.split("/").pop() ?? "";
  const pageTitle = last ? last.charAt(0).toUpperCase() + last.slice(1) : "";

  const [unit, setUnit] = useState<string>("sats");
  const [showFilter, setShowFilter] = useState(false);
  const [isCapacityOpen, setIsCapacityOpen] = useState(false);
  const [isDateOpen, setIsDateOpen] = useState(false);
  const [isStateOpen, setIsStateOpen] = useState(false);

  // Local filter states
  const [capacityOperator, setCapacityOperator] = useState<"gte" | "lte" | "eq" | "">("");
  const [capacityValue, setCapacityValue] = useState<string>("");
  const [dateFrom, setDateFrom] = useState<string>("");
  const [dateTo, setDateTo] = useState<string>("");
  const [paymentType, setPaymentType] = useState<"all" | "incoming" | "outgoing" | "">("");

  if (!pageTitle) return null;

  const units = ["sats", "BTC", "USD"] as const;

  const handleApply = () => {
    const filters: HeaderFilters = {};
    if (paymentType) filters.paymentType = paymentType;
    if (capacityOperator) filters.operator = capacityOperator;
    if (capacityValue) filters.value = Number(capacityValue);
    if (dateFrom) filters.from = dateFrom;
    if (dateTo) filters.to = dateTo;
    onApplyFilters?.(filters);
    setShowFilter(false);
  };

  return (
    <div className="flex items-center justify-between mb-2 mt-4 font-clash-grotesk text-grey-dark">
      <h1 className="text-3xl font-medium">{pageTitle}</h1>

      <div className="flex items-center gap-4">
        <button className="flex items-center gap-2 text-sm bg-[#204ECF] rounded-[50px] text-[#F1F9FF] px-[15px] py-[10px] font-[500]">
          <Image src={ExportData} alt="Export Data" />
          <p>Export Data</p>
        </button>

        <button
          className="flex items-center gap-2 text-sm border-[1px] font-[500] border-[#D4D4D4] bg-[#F7F7F7] rounded-[50px] text-[#294459] px-[25px] py-[10px]"
          onClick={() => setShowFilter(true)}
        >
          <Image src={Filter} alt="Filter" />
          <p>Filter</p>
        </button>
        

        {showFilter && (
          <div className="fixed inset-0 z-[9999] flex justify-end">
            {/* Overlay */}
            <div
              className="absolute inset-0 bg-black/20"
              onClick={() => setShowFilter(false)}
            />
            {/* Drawer */}
            <div className="relative h-full w-full max-w-[420px] bg-white shadow-xl flex flex-col">
              <div className="flex items-center justify-between p-6">
                <div className="flex items-center gap-2">
                    <Image src={Filter} alt="Filter" className="bg-[#F7F7F7] 
                              border-[1px] 
                               border-[#D4D4D4]
                              rounded-[8px]
                              p-[2px]
                              w-[24px]" />
                  <span className="font-[500] text-lg">Filter</span>
                </div>
                <div 
                onClick={() => setShowFilter(false)}
                className="">
                    <Image src={Close} alt="Close"/>
               
                    </div>
              </div>
              {/* Drawer content goes here */}
              <div className="flex-1 overflow-y-auto px-6">
                {/* Example content */}
                <div className="flex justify-between items-center">
                <button className="bg-[#F6F6F6] my-[20px] h-9 rounded-full px-4
                      font-[500] text-[15px] flex
                      justify-center items-center gap-2">
                    <Image src={Add} alt="Add"/>
                  <span className="text-sm font-medium">Add Filter</span>
                </button>
                <Button className="h-9 rounded-full bg-[#204ECF] text-[#F1F9FF] px-4" onClick={handleApply}>
                  Apply Filter
                 </Button>

                 </div>

                <div className="mb-6">
                  <button
                    type="button"
                    onClick={() => setIsCapacityOpen((v) => !v)}
                    className="bg-[#EFF6FF] rounded-[20px] flex justify-between px-[20px] py-[15px] w-full text-left"
                  >
                 <p className="text-sm font-semibold">Capacity</p>
                 <Image src={CapacityIcon} alt="Capicity Icon"/>
                 </button>
                  {isCapacityOpen && (
                    <div className="space-y-2 mt-4">
                    <select aria-label="Capacity" className="w-full rounded-lg border border-[#D4D4D4] bg-white px-2 py-4 text-sm outline-none" 
                    value={capacityOperator} 
                    onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setCapacityOperator(e.target.value as "gte" | "lte" | "eq" | "") }>
                      <option value="">Select operator</option>
                      <option value="gte">Is greater than or equal to</option>
                      <option value="lte">Is less than or equal to</option>
                      <option value="eq">Is exactly</option>
                    </select>
                    <input
                      type="number"
                      placeholder="5,000"
                      className="w-full rounded-lg border border-[#D4D4D4] bg-white px-3 py-4 text-sm outline-none"
                      value={capacityValue}
                      onChange={(e) => setCapacityValue(e.target.value)}
                    />
                  </div>
                  )}
                </div>
                <div>
                  <button
                    type="button"
                    onClick={() => setIsDateOpen((v) => !v)}
                    className="bg-[#EFF6FF] rounded-[20px] flex justify-between px-[20px] py-[15px] mb-[20px] w-full text-left"
                  >
                 <p className="text-sm font-semibold">Date Range</p>
                 <Image src={CapacityIcon} alt="Capicity Icon"/>
                 </button>
                  {isDateOpen && (
                    <div className="flex flex-col gap-2">
                      <label htmlFor="From" className="text-[#727A86] text-[15px] font-[400]">
                          From
                      <input
                        type="date"
                        className="w-full border rounded-lg px-[10px] py-2 "
                        placeholder="Select"
                        value={dateFrom}
                        onChange={(e) => setDateFrom(e.target.value)}
                      />
                      </label>
                      <label htmlFor="From" className="text-[#727A86] text-[15px] font-[400]">
                          To
                      <input
                        type="date"
                        className="w-full border rounded-lg px-[10px] py-2 "
                        placeholder="Select"
                        value={dateTo}
                        onChange={(e) => setDateTo(e.target.value)}
                      />
                      </label>
                    </div>
                  )}
                </div>

                <div className="mt-6">
                  <button
                    type="button"
                    onClick={() => setIsStateOpen((v) => !v)}
                    className="bg-[#EFF6FF] rounded-[20px] flex justify-between px-[20px] py-[15px] w-full text-left"
                  >
                 <p className="text-sm font-semibold">State</p>
                 <Image src={CapacityIcon} alt="Capicity Icon"/>
                 </button>
                  {isStateOpen && (
                    <div className="space-y-2 mt-4">
                    <select aria-label="Payment Type" className="w-full rounded-lg border border-[#D4D4D4] bg-white px-2 py-4 text-sm outline-none" value={paymentType} onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setPaymentType(e.target.value as "all" | "incoming" | "outgoing" | "") }>
                      <option value="">Select type</option>
                      <option value="all">All Payments</option>
                      <option value="incoming">Incoming Payments</option>
                      <option value="outgoing">Outgoing Payments</option>
                    </select>
                  </div>
                  )}
                </div>

                
                
              </div>
            </div>
          </div>
        )}

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="outline"
              size="sm"
              className="flex items-center gap-2 text-sm border-[1px] font-[500] border-[#D4D4D4] bg-[#F7F7F7] rounded-[50px] text-[#294459] px-[25px] py-[20px]"
            >
              {unit} <ChevronDown className="ml-1 h-6 w-6" />
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

