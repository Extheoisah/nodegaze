"use client";

import * as React from "react";
import {
  ColumnDef,
  ColumnFiltersState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  SortingState,
  useReactTable,
  VisibilityState,
} from "@tanstack/react-table";
import { ChevronDown } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import Link from "next/link";

const data: Channel[] = [
  {
    id: "022047c869809c747ae69603c4ed178c4c3cb899c8e5370a5d8684b0e2b508e058",
    channelName: "90258lx1823x0",
    state: "active",
    inboundBalance: 5000000,
    outboundBalance: 5000000,
    date: "15th Feb, 2025",
    uptime: "Very Good",
  },
  {
    id: "022047c869809c747ae69603c4ed178c4c3cb899c8e5370a5d8684b0e2b508e058",
    channelName: "45789abc4567x1",
    state: "active",
    inboundBalance: 3500000,
    outboundBalance: 2800000,
    date: "14th Feb, 2025",
    uptime: "Good",
  },
  {
    id: "022047c869809c747ae69603c4ed178c4c3cb899c8e5370a5d8684b0e2b508e058",
    channelName: "78901def7890x2",
    state: "inactive",
    inboundBalance: 1200000,
    outboundBalance: 4500000,
    date: "13th Feb, 2025",
    uptime: "Poor",
  },
  {
    id: "022047c869809c747ae69603c4ed178c4c3cb899c8e5370a5d8684b0e2b508e058",
    channelName: "12345ghi1234x3",
    state: "active",
    inboundBalance: 7500000,
    outboundBalance: 1500000,
    date: "12th Feb, 2025",
    uptime: "Very Good",
  },
  {
    id: "022047c869809c747ae69603c4ed178c4c3cb899c8e5370a5d8684b0e2b508e058",
    channelName: "67890jkl6789x4",
    state: "pending",
    inboundBalance: 2100000,
    outboundBalance: 3400000,
    date: "11th Feb, 2025",
    uptime: "Good",
  },
];

export type Channel = {
  id: string;
  channelName: string;
  state: "active" | "inactive" | "pending";
  inboundBalance: number;
  outboundBalance: number;
  date: string;
  uptime: "Very Good" | "Good" | "Poor";
};

export const columns: ColumnDef<Channel>[] = [
  {
    accessorKey: "channelName",
    header: "Channel Name",
    cell: ({ row }) => {
      const channelId = row.original.id;
      return (
        <Link
          href={`/channels/${channelId}`}
          className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer"
        >
          {row.getValue("channelName")}
        </Link>
      );
    },
  },
  {
    accessorKey: "state",
    header: "State",
    cell: ({ row }) => {
      const state = row.getValue("state") as string;
      const getStateStyle = (state: string) => {
        switch (state) {
          case "active":
            return "bg-green-100 text-green-800 border-green-200";
          case "inactive":
            return "bg-red-100 text-red-800 border-red-200";
          case "pending":
            return "bg-yellow-100 text-yellow-800 border-yellow-200";
          default:
            return "bg-gray-100 text-gray-800 border-gray-200";
        }
      };
      return (
        <span
          className={`inline-flex items-center px-2.5 py-0.5 rounded-full font-normal border ${getStateStyle(
            state
          )}`}
        >
          {state.charAt(0).toUpperCase() + state.slice(1)}
        </span>
      );
    },
  },
  {
    accessorKey: "inboundBalance",
    header: "Inbound Balance",
    cell: ({ row }) => {
      const balance = row.getValue("inboundBalance") as number;
      const formatted = new Intl.NumberFormat("en-US").format(balance);
      return <div className="text-grey-dark">{formatted} sats</div>;
    },
  },
  {
    accessorKey: "outboundBalance",
    header: "Outbound Balance",
    cell: ({ row }) => {
      const balance = row.getValue("outboundBalance") as number;
      const formatted = new Intl.NumberFormat("en-US").format(balance);
      return <div className="text-grey-dark">{formatted} sats</div>;
    },
  },
  {
    accessorKey: "date",
    header: "Date",
    cell: ({ row }) => (
      <div className="text-grey-dark">{row.getValue("date")}</div>
    ),
  },
  {
    accessorKey: "uptime",
    header: "Uptime",
    cell: ({ row }) => {
      const uptime = row.getValue("uptime") as string;
      const getUptimeColor = (uptime: string) => {
        switch (uptime) {
          case "Very Good":
            return "text-green-600";
          case "Good":
            return "text-yellow-600";
          case "Poor":
            return "text-red-600";
          default:
            return "text-gray-600";
        }
      };
      const getProgressPercentage = (uptime: string) => {
        switch (uptime) {
          case "Very Good":
            return 90;
          case "Good":
            return 70;
          case "Poor":
            return 30;
          default:
            return 0;
        }
      };

      const getStrokeColor = (uptime: string) => {
        switch (uptime) {
          case "Very Good":
            return "#33CE6D";
          case "Good":
            return "#EAB308";
          case "Poor":
            return "#EF4444";
          default:
            return "#D1D5DB";
        }
      };

      const percentage = getProgressPercentage(uptime);
      //   const circumference = 2 * Math.PI * 16; // radius = 16
      //   const strokeDasharray = circumference;
      //   const strokeDashoffset = circumference - (percentage / 100) * circumference;

      return (
        <div className="flex items-center gap-3">
          <span className={`font-normal ${getUptimeColor(uptime)}`}>
            {uptime}
          </span>
          <div className="relative w-8 h-8">
            <svg className="w-8 h-8 transform -rotate-90" viewBox="0 0 36 36">
              {/* Background circle */}
              <path
                d="M18 2.0845
                  a 15.9155 15.9155 0 0 1 0 31.831
                  a 15.9155 15.9155 0 0 1 0 -31.831"
                fill="none"
                stroke="#E5E7EB"
                strokeWidth="3"
              />
              {/* Progress circle */}
              <path
                d="M18 2.0845
                  a 15.9155 15.9155 0 0 1 0 31.831
                  a 15.9155 15.9155 0 0 1 0 -31.831"
                fill="none"
                strokeWidth="3"
                stroke={getStrokeColor(uptime)}
                strokeDasharray={`${percentage}, 100`}
                strokeLinecap="round"
              />
            </svg>
          </div>
        </div>
      );
    },
  },
  {
    id: "actions",
    header: "",
    enableHiding: false,
    cell: ({ row }) => {
      const channelId = row.original.id;
      return (
        <Link href={`/channels/${channelId}`} className="cursor-pointer">
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

export function DataTable() {
  const [sorting, setSorting] = React.useState<SortingState>([]);
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  );
  const [columnVisibility, setColumnVisibility] =
    React.useState<VisibilityState>({});

  const table = useReactTable({
    data,
    columns,
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onColumnVisibilityChange: setColumnVisibility,
    state: {
      sorting,
      columnFilters,
      columnVisibility,
    },
  });

  return (
    <div className="w-full">
      <div className="rounded-xl border">
        <Table className="bg-white">
          <TableHeader>
            {/* All Channels header row */}
            <TableRow className="border-b">
              <TableHead colSpan={columns.length} className="py-6 px-4">
                <div className="flex items-center gap-3">
                  <h1 className="text-2xl font-medium text-grey-dark">
                    All Channels
                  </h1>
                  <span className="bg-cerulean-blue text-grey-dark px-3 py-1 rounded-2xl text-sm font-medium">
                    {data.length}
                  </span>
                </div>
              </TableHead>
            </TableRow>
            {/* Column headers */}
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead
                      key={header.id}
                      className="text-grey-table-header font-medium text-sm py-3 px-4"
                    >
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() && "selected"}
                >
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
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      <div className="flex items-center justify-end space-x-2 py-4">
        <div className="space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => table.previousPage()}
            disabled={!table.getCanPreviousPage()}
          >
            Previous
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => table.nextPage()}
            disabled={!table.getCanNextPage()}
          >
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}
