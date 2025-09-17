"use client";

import * as React from "react";
import { useState } from "react";
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { Button } from "./ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import Link from "next/link";
import { ChevronDown, Copy } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export type Payment = {
  state: string;
  payment_type: string;
  amount_sat: number;
  amount_usd: number;
  routing_fee: number;
  creation_time: {
    secs_since_epoch: number;
    nanos_since_epoch: number;
  } | number;
  invoice: string;
  payment_hash: string;
  completed_at: number;
};

export type TableFilters = {
  paymentState?: "settled" | "failed" | "pending";
  operator?: "gte" | "lte" | "eq";
  value?: number;
  from?: string;
  to?: string;
};

export function DataTable({
  payments,
  setPayments,
  selectedState,
  filters,
}: {
  payments: Payment[];
  setPayments: React.Dispatch<React.SetStateAction<Payment[]>>;
  selectedState: string;
  filters?: TableFilters;
}) {
  const [page, setPage] = React.useState(1);
  const [totalPages, setTotalPages] = React.useState(1);
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string>("");

  // Fetch payments from endpoint
  React.useEffect(() => {
    async function fetchPayments() {
      try {
        setIsLoading(true);
        setError("");
        const params = new URLSearchParams();
        params.set("page", String(page));
        params.set("per_page", "10");
        // Map UI selections to backend query params:
        // - selectedState (incoming/outgoing) -> payment_types
        // - filters.paymentState (settled/failed/pending) -> states
        // Both can be present at the same time
        if (selectedState && selectedState !== "all") {
          params.set("payment_types", selectedState);
        }
        if (filters?.paymentState) {
          params.set("states", filters.paymentState);
        }
        if (filters?.operator) params.set("operator", filters.operator);
        if (typeof filters?.value === "number") params.set("value", String(filters.value));
        if (filters?.from) params.set("from", filters.from);
        if (filters?.to) params.set("to", filters.to);

        const res = await fetch(`/api/payments?${params.toString()}`);
        console.log("API URL:", `/api/payments?${params.toString()}`);
        console.log("API Status:", res.status);
        
        if (!res.ok) {
          const errorText = await res.text();
          console.error("API Error Response:", errorText);
          throw new Error(`API Error: ${res.status} - ${errorText}`);
        }
        
        const data = await res.json();
        console.log("API RESPONSE:", data);
        setPayments(data?.data?.items || []);
        setTotalPages(data.pagination?.total_pages || 1);
      } catch (err) {
        console.error("Failed to fetch payments:", err);
        // surface a simple error state similar to channels table
        setError(
          err instanceof Error ? err.message : "Something went wrong while fetching payments"
        );
        // optional: clear payments on error
        // setPayments([]);
      } finally {
        setIsLoading(false);
      }
    }
    fetchPayments();
  }, [page, selectedState, filters, setPayments]);

  const [copied, setCopied] = useState<string | null>(null);

  // Clipboard function
  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(text);
      setTimeout(() => setCopied(null), 1200);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };
  
  const columns: ColumnDef<Payment>[] = [
    {
      accessorKey: "state",
      header: "State",
      cell: ({ row }) => {
        const state = row.getValue("state") as string;
        const getStateStyle = (state: string) => {
          switch (state) {
            case "Settled":
              return "bg-green-100 text-green-800 border-green-200";
            case "Failed":
              return "bg-red-100 text-red-800 border-red-200";
            case "Pending":
              return "bg-yellow-100 text-yellow-800 border-yellow-200";
            default:
              return "";
          }
        };
        return (
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-full font-normal border ${getStateStyle(
              state
            )}`}
          >
            {state ? state.charAt(0).toUpperCase() + state.slice(1) : "Unknown"}
          </span>
        );
      },
    },
    {
  accessorKey: "payment_type",
  header: "Type",
  cell: ({ row }) => {
    // Use optional chaining and fallback to "Unknown" if type is missing
    const type = row.getValue("payment_type") as string | undefined;
    const getTypeStyle = (type?: string) => {
      switch (type) {
        case "Outgoing":
          return "bg-gray-100 text-gray-600 border-gray-200";
        case "Incoming":
          return "bg-gray-100 text-gray-600 border-gray-200";
        case "Forwarded":
          return "bg-gray-100 text-gray-600 border-gray-200";
        default:
          return "bg-gray-100 text-gray-600 border-gray-200";
      }
    };
    return (
      <span
        className={`inline-flex items-center px-2.5 py-0.5 rounded-full font-normal border ${getTypeStyle(
          type
        )}`}
      >
        {type ? type.charAt(0).toUpperCase() + type.slice(1) : "Unknown"}
      </span>
    );
  },
},
    {
      accessorKey: "amount_sat",
      header: "Amount (sats)",
      cell: ({ row }) => {
        const balance = row.getValue("amount_sat") as number;
        const formatted = new Intl.NumberFormat("en-US").format(balance);
        return <div className="text-grey-dark">{formatted} sats</div>;
      },
    },
    {
      accessorKey: "amount_usd",
      header: "Amount (USD)",
      cell: ({ row }) => {
        const balance = row.getValue("amount_usd") as number;
        const formatted = new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" }).format(balance);
        return <div className="text-grey-dark">{formatted}</div>;
      },
    },
    {
      accessorKey: "routing_fee",
      header: "Routing Fee (sats)",
      cell: ({ row }) => {
        const fee = row.getValue("routing_fee") as number;
        const formatted = new Intl.NumberFormat("en-US").format(fee);
        return <div className="text-grey-dark">{formatted} sats</div>;
      },
    },
    {
      accessorKey: "invoice",
      header: "Invoice",
      cell: ({ row }) => {
        const invoice = row.getValue("invoice") as string;
        const truncated = invoice.length > 16 ? `${invoice.slice(0, 14)}...` : invoice;
        return (
          <div className="flex items-center gap-1 relative">
            <div
              // href={`/payments/${invoice}`}
              className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer truncate"
              title={invoice}
            >
              {truncated}
            </div>
            <Button
              variant="link"
              size="sm"
              onClick={() => copyToClipboard(invoice)}
              className="p-1 h-6 w-6"
              type="button"
            >
              <Copy className="h-3 w-3" />
            </Button>
            {copied === invoice && (
              <span className="text-xs text-green-600 ml-1 absolute top-[5px] right-[40px]">Copied!</span>
            )}
          </div>
        );
      },
    },
    {
      accessorKey: "creation_time",
      header: "Date",
      cell: ({ row }) => {
        const creation = row.getValue("creation_time") as
          | number
          | { secs_since_epoch: number }
          | undefined
          | null;
        const secsSinceEpoch =
          typeof creation === "number"
            ? creation
            : (creation && typeof creation === "object" && "secs_since_epoch" in creation
                ? (creation as { secs_since_epoch: number }).secs_since_epoch
                : undefined);
        const date = secsSinceEpoch
          ? new Date(secsSinceEpoch * 1000).toLocaleDateString()
          : "-";
        return <div className="text-grey-dark">{date}</div>;
      },
    },
    {
      id: "actions",
      accessorKey: "payment_hash",
      header: "",
      enableHiding: false,
      cell: ({ row }) => {
        const hash = row.original.payment_hash;
        return (
          <Link href={`/payments/${hash}`} className="cursor-pointer">
            <Button
              variant="outline"
              className="h-8 w-8 p-0 text-grey-dark rounded-[8px] cursor-pointer"
            >
              <ChevronDown className="h-4 w-4 rotate-[-90deg]" />
            </Button>
          </Link>
        );
      },
    },
  ];
  
  const table = useReactTable({
    data: payments,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    state: {},
  });

  return (
    <div className="w-full">
      <div className="rounded-xl border">
        <Table className="bg-white overflow-hidden">
          <TableHeader>
            <TableRow className="border-b">
              <TableHead colSpan={columns.length} className="py-6 px-4">
                <div className="flex items-center gap-3">
                  <h1 className="text-2xl font-medium text-grey-dark">
                    All Payments
                  </h1>
                  <span className="bg-cerulean-blue text-grey-dark px-3 py-1 rounded-2xl text-sm font-medium">
                    {payments.length}
                  </span>
                </div>
              </TableHead>
            </TableRow>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    className="text-grey-table-header font-medium text-sm py-3 px-4"
                  >
                    {flexRender(
                      header.column.columnDef.header,
                      header.getContext()
                    )}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={columns.length} className="py-6 text-center text-grey-accent">
                  Loading payments...
                </TableCell>
              </TableRow>
            ) : error ? (
              <TableRow>
                <TableCell colSpan={columns.length} className="py-6 text-center text-grey-accent">
                  {"No Payments Available..."}
                </TableCell>
              </TableRow>
            ) : table.getRowModel().rows.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow key={row.id}>
                  {row.getVisibleCells().map((cell) => (
                    <TableCell
                      key={cell.id}
                      className="px-4 py-6 text-sm font-normal"
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No Payments Available...
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      <div className="flex items-center justify-end space-x-6 py-4">
        <span className="text-base text-gray-700 font-medium">
          Page {page} of {totalPages}
        </span>
        <div className="flex items-center space-x-1">
          {Array.from({ length: Math.min(totalPages, 6) }, (_, i) => {
            const pageNum = i + 1;
            return (
              <button
                key={pageNum}
                onClick={() => setPage(pageNum)}
                className={`px-3 py-1 rounded-lg border transition-colors duration-150 text-base font-medium
            ${
              page === pageNum
                ? "border-blue-500 text-blue-600 bg-white shadow-[0_0_0_2px_#2563eb_inset]"
                : "border-transparent text-gray-700 hover:bg-gray-100"
            }
          `}
                disabled={page === pageNum}
              >
                {pageNum}
              </button>
            );
          })}
          {totalPages > 6 && <span className="px-2 text-base">...</span>}
        </div>
        <span className="text-base text-gray-700 font-medium">Go to page</span>
        <div className="relative flex items-center">
          <Select
            value={page.toString()}
            onValueChange={(val) => setPage(Number(val))}
            disabled={totalPages === 1}
          >
            <SelectTrigger className="w-[60px] border border-gray-300 rounded-lg px-[30px] py-1 text-base font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white pr-8">
              <SelectValue className="w-[60px] flex justify-center">
                {page.toString().padStart(2, "0")}
              </SelectValue>
            </SelectTrigger>
            <SelectContent className="w-[60px] min-w-[60px] max-w-[60px] p-0">
              {Array.from({ length: totalPages }, (_, i) => (
                <SelectItem
                  className="w-[60px] min-w-[60px] max-w-[60px] outline-none flex justify-center text-center"
                  key={i + 1}
                  value={(i + 1).toString()}
                >
                  {(i + 1).toString().padStart(2, "0")}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  );
}
