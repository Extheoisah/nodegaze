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
import { NodeActions } from "@/components/ui/node-dropdown-menu";

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
import { ChevronDown } from "lucide-react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export type Node = {
  id: number;
  status: string;
  Node_alias: string;
  Node_PubKey: string;
  inbound_balance: number;
  outbound_balance: number;
  Date_connected: string;
  Last_synced: string;
};

export function DataTable() {
  const { data: session, status } = useSession();
  console.log("Session:", session);
  console.log("Status:", status);

  // ---------- DUMMY DATA ----------
  const dummyNodes: Node[] = [
    {
      id: 1,
      status: "Online",
      Node_alias: "AlphaNode",
      Node_PubKey:
        "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
      inbound_balance: 50000,
      outbound_balance: 30000,
      Date_connected: "2025-01-01",
      Last_synced: "10:30 AM",
    },
    {
      id: 2,
      status: "Offline",
      Node_alias: "BetaNode",
      Node_PubKey:
        "03a34b3ef57a8c2c57e3d2f3ad5cfa1b7f4e5bc3a72c5efb8d63b39b7c2b4f4b2c",
      inbound_balance: 20000,
      outbound_balance: 10000,
      Date_connected: "2025-01-10",
      Last_synced: "11:15 AM",
    },
    {
      id: 3,
      status: "Online",
      Node_alias: "GammaNode",
      Node_PubKey:
        "023a54f7c1d5a6e3b3f1d7e8a9b2c5d3f4a1b6e2c3d7a8f9b4c5d6e7f8a9b2c3d4",
      inbound_balance: 75000,
      outbound_balance: 45000,
      Date_connected: "2025-02-01",
      Last_synced: "09:45 AM",
    },
  ];

  const [nodes, setNodes] = React.useState<Node[]>(dummyNodes);
  const [sorting, setSorting] = React.useState<SortingState>([]);
  const [openDropdownId, setOpenDropdownId] = React.useState<number | null>(
    null
  );

  const columns: ColumnDef<Node>[] = [
    {
      accessorKey: "status",
      header: "Status",
      cell: ({ row }) => {
        const status = row.getValue("status") as string;
        const getStatusStyle = (status: string) => {
          switch (status) {
            case "Online":
              return "bg-green-100 text-green-800 border-green-200";
            case "Offline":
              return "bg-gray-100 text-red-800 border-red-200";
          }
        };
        return (
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-full font-normal border ${getStatusStyle(
              status
            )}`}
          >
            {status
              ? status.charAt(0).toUpperCase() + status.slice(1)
              : "Unknown"}
          </span>
        );
      },
    },
    {
      accessorKey: "Node_alias",
      header: "Node Alias",
      cell: ({ row }) => {
        const NodeId = row.original.id;
        return (
          <Link
            href={`/nodes/${NodeId}`}
            className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer"
          >
            {row.getValue("Node_alias")}
          </Link>
        );
      },
    },
    {
      accessorKey: "Node_PubKey",
      header: "Node Pub Key",
      cell: ({ row }) => {
        const NodePubkey = row.original.id;
        const pubkey = row.getValue("Node_PubKey") as string;
        const truncated =
          pubkey.length > 16 ? `${pubkey.slice(0, 25)}...` : pubkey;
        return (
          <Link
            href={`/nodes/${NodePubkey}`}
            className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer truncate"
            title={pubkey}
          >
            {truncated}
          </Link>
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
      accessorKey: "Date_connected",
      header: "Date Connected",
      cell: ({ row }) => (
        <div className="text-grey-dark">{row.getValue("Date_connected")}</div>
      ),
    },
    {
      accessorKey: "Last_synced",
      header: "Last Synced",
      cell: ({ row }) => (
        <div className="text-grey-dark">{row.getValue("Last_synced")}</div>
      ),
    },
    {
      id: "actions",
      header: "",
      enableHiding: false,
      cell: ({ row }) => (
        <NodeActions
          nodeId={row.original.id}
          openId={openDropdownId}
          setOpenId={setOpenDropdownId}
        />
      ),
    },
  ];

  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  );
  const [columnVisibility, setColumnVisibility] =
    React.useState<VisibilityState>({});

  const [page, setPage] = React.useState(1);
  const [totalPages, setTotalPages] = React.useState(1);

  //   const fetchChannels = async (pageNum = 1) => {
  //     setIsLoading(true);
  //     setError("");
  //     try {
  //       const res = await fetch(`/api/channels?page=${pageNum}&per_page=10`);
  //       const result = await res.json();
  //       console.log("API RESPONSE:", result);

  //       if (!res.ok) {
  //         const errorMessage =
  //           typeof result.error === "string"
  //             ? result.error
  //             : result.error?.message ||
  //               JSON.stringify(result.error) ||
  //               "Failed to fetch channels";
  //         throw new Error(errorMessage);
  //       }

  //       const apiItems = (result?.data?.items ?? []) as ApiNode[];
  //       setTotalPages(result?.data?.total_pages || 1);

  //       if (apiItems.length === 0) {
  //         setChannels([]);
  //         setError("No channels available...");
  //         return;
  //       }

  //       const transformed: Node[] = apiItems.map((item) => ({
  //         id: item.chan_id,
  //         status: item.channel_state === "active" ? "Online" : "Offline",
  //         Node_alias: item.alias ?? String(item.chan_id),
  //         Node_PubKey: String(item.chan_id),
  //         inbound_balance: Number(item.remote_balance ?? 0),
  //         outbound_balance: Number(item.local_balance ?? 0),
  //         Date_connected: new Date(
  //           (item.last_update ?? 0) * 1000
  //         ).toLocaleDateString(),
  //         Last_synced: new Date(
  //           (item.last_update ?? 0) * 1000
  //         ).toLocaleTimeString(),
  //       }));

  //       setChannels(transformed);
  //     } catch (err) {
  //       setError(err instanceof Error ? err.message : "Something went wrong");
  //     } finally {
  //       setIsLoading(false);
  //     }
  //   };

  //   React.useEffect(() => {
  //     fetchChannels(page);
  //   }, [page]);

  const table = useReactTable({
    data: nodes,
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
        <Table className="bg-white overflow-hidden">
          <TableHeader>
            <TableRow className="border-b">
              <TableHead colSpan={columns.length} className="py-6 px-4">
                <div className="flex items-center gap-3">
                  <h1 className="text-2xl font-medium text-grey-dark">
                    All Connected Nodes
                  </h1>
                  <span className="bg-cerulean-blue text-grey-dark px-3 py-1 rounded-2xl text-sm font-medium">
                    {nodes.length}
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
            {table.getRowModel().rows.length ? (
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
      {/* <div className="flex items-center justify-end space-x-2 py-4">
        <Button
          variant="outline"
          size="sm"
          onClick={() => setPage((p) => Math.max(1, p - 1))}
          disabled={page === 1}
        >
          Previous
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
          disabled={page === totalPages}
        >
          Next
        </Button>
      </div> */}

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
