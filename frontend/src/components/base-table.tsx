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
import { useSession } from "next-auth/react";

type ApiChannel = {
  chan_id: number;
  alias: string | null;
  channel_state: string | null;
  private: boolean;
  remote_balance: number;
  local_balance: number;
  capacity: number;
  last_update: number;
  uptime: number;
};

export type Channel = {
  id: number;
  channel_name: string;
  state: string;
  inbound_balance: number;
  outbound_balance: number;
  last_updated: string;
  uptime: number;
};

export const columns: ColumnDef<Channel>[] = [
  {
    accessorKey: "channel_name",
    header: "Channel Name",
    cell: ({ row }) => {
      const channelId = row.original.id;
      return (
        <Link
          href={`/channels/${channelId}`}
          className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer"
        >
          {row.getValue("channel_name")}
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
          {state ? state.charAt(0).toUpperCase() + state.slice(1) : "Unknown"}
        </span>
      );
    },
  },
  {
    accessorKey: "inbound_balance",
    header: "Inbound Balance",
    cell: ({ row }) => {
      const balance = row.getValue("inbound_balance") as number;
      const formatted = new Intl.NumberFormat("en-US").format(balance);
      return <div className="text-grey-dark">{formatted} sats</div>;
    },
  },
  {
    accessorKey: "outbound_balance",
    header: "Outbound Balance",
    cell: ({ row }) => {
      const balance = row.getValue("outbound_balance") as number;
      const formatted = new Intl.NumberFormat("en-US").format(balance);
      return <div className="text-grey-dark">{formatted} sats</div>;
    },
  },
  {
    accessorKey: "last_updated",
    header: "Last Updated",
    cell: ({ row }) => (
      <div className="text-grey-dark">{row.getValue("last_updated")}</div>
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
      // const getProgressPercentage = (uptime: string) => {
      //   switch (uptime) {
      //     case "Very Good":
      //       return 90;
      //     case "Good":
      //       return 70;
      //     case "Poor":
      //       return 30;
      //     default:
      //       return 0;
      //   }
      // };

      // const getStrokeColor = (uptime: string) => {
      //   switch (uptime) {
      //     case "Very Good":
      //       return "#33CE6D";
      //     case "Good":
      //       return "#EAB308";
      //     case "Poor":
      //       return "#EF4444";
      //     default:
      //       return "#D1D5DB";
      //   }
      // };

      // const percentage = getProgressPercentage(uptime);
      //   const circumference = 2 * Math.PI * 16; // radius = 16
      //   const strokeDasharray = circumference;
      //   const strokeDashoffset = circumference - (percentage / 100) * circumference;

      return (
        <div className="flex items-center gap-3">
          <span className={`font-normal ${getUptimeColor(uptime)}`}>
            {uptime}
          </span>
          <div className="relative w-8 h-8">
            {/* <svg className="w-8 h-8 transform -rotate-90" viewBox="0 0 36 36">
              Background circle
              <path
                d="M18 2.0845
                  a 15.9155 15.9155 0 0 1 0 31.831
                  a 15.9155 15.9155 0 0 1 0 -31.831"
                fill="none"
                stroke="#E5E7EB"
                strokeWidth="3"
              />
              Progress circle
              <path
                d="M18 2.0845
                  a 15.9155 15.9155 0 0 1 0 31.831
                  a 15.9155 15.9155 0 0 1 0 -31.831"
                fill="none"
                strokeWidth="3"
                stroke={getStrokeColor(uptime)}
                strokeDasharray={`${getProgressPercentage}, 100`}
                strokeLinecap="round"
              />
            </svg> */}
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
  const { data: session, status } = useSession();
  console.log("Session:", session);
  console.log("Status:", status);

  const [channels, setChannels] = React.useState<Channel[]>([]);
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string>("");

  const [sorting, setSorting] = React.useState<SortingState>([]);
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  );
  const [columnVisibility, setColumnVisibility] =
    React.useState<VisibilityState>({});

  const [page, setPage] = React.useState(1);
  const [totalPages, setTotalPages] = React.useState(1);

  const fetchChannels = async (pageNum = 1) => {
    setIsLoading(true);
    setError("");
    try {
      const res = await fetch(`/api/channels?page=${pageNum}&per_page=10`);
      const result = await res.json();
      console.log("API RESPONSE:", result)

      if (!res.ok) {
        const errorMessage =
          typeof result.error === "string"
            ? result.error
            : result.error?.message ||
              JSON.stringify(result.error) ||
              "Failed to fetch channels";
        throw new Error(errorMessage);
      }

      const apiItems = (result?.data?.items ?? []) as ApiChannel[];
      setTotalPages(result?.data?.total_pages || 1); 

      if (apiItems.length === 0) {
        setChannels([]);
        setError("No channels available...");
        return;
      }

      const transformed: Channel[] = apiItems.map((item) => {
        const rawState = (item.channel_state ?? "").toString().toLowerCase();
        const state: Channel["state"] =
          rawState === "active" ||
          rawState === "inactive" ||
          rawState === "pending"
            ? (rawState as Channel["state"])
            : "Disabled";
        // Uptime as string category for display
        // const uptimeSeconds = typeof item.uptime === "number" ? item.uptime : 0;
        // const uptimePercentage = (uptimeSeconds / 86400) * 100;
        // let uptimeCategory: string;
        // if (uptimePercentage >= 90) uptimeCategory = "Very Good";
        // else if (uptimePercentage >= 70) uptimeCategory = "Good";
        // else uptimeCategory = "Poor";

        return {
          id: item.chan_id,
          channel_name:
            item.alias && item.alias.trim() !== ""
              ? item.alias
              : String(item.chan_id),
          state,
          inbound_balance: Number(item.remote_balance ?? 0),
          outbound_balance: Number(item.local_balance ?? 0),
          last_updated: new Date(
            (item.last_update ?? 0) * 1000
          ).toLocaleString(),
          uptime: item.uptime,
        };
      });


      setChannels(transformed);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Something went wrong");
    } finally {
      setIsLoading(false);
    }
  };

  // React.useEffect(() => {
  //   fetchAlias(); // fetch alias once
  // }, []);

  React.useEffect(() => {
    fetchChannels(page);
  }, [page]); 



  const table = useReactTable({
    data: channels,
    columns,
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onColumnVisibilityChange: setColumnVisibility,
    state: { sorting, columnFilters, columnVisibility },
  });

  return (
    <div className="w-full">
      <div className="rounded-xl border">
        <Table className="bg-white">
          <TableHeader>
            <TableRow className="border-b">
              <TableHead colSpan={columns.length} className="py-6 px-4">
                <div className="flex items-center gap-3">
                  <h1 className="text-2xl font-medium text-grey-dark">
                    All Channels
                  </h1>
                  <span className="bg-cerulean-blue text-grey-dark px-3 py-1 rounded-2xl text-sm font-medium">
                    {channels.length}
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
                <TableCell
                  colSpan={columns.length}
                  className="py-6 text-center text-grey-accent"
                >
                  Loading channels...
                </TableCell>
              </TableRow>
            ) : error ? (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="py-6 text-center text-grey-accent"
                >
                  {error}
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
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      <div className="flex items-center justify-end space-x-2 py-4">
        <Button
          variant="outline"
          size="sm"
          // onClick={() => table.previousPage()}
          // disabled={!table.getCanPreviousPage()}
          onClick={() => setPage((p) => Math.max(1, p - 1))}
          disabled={page === 1}
        >
          Previous
        </Button>
        <Button
          variant="outline"
          size="sm"
          // onClick={() => table.nextPage()}
          // disabled={!table.getCanNextPage()}
          onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
          disabled={page === totalPages}
        >
          Next
        </Button>
      </div>
    </div>
  );
}
