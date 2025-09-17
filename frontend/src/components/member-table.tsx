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

import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useSession } from "next-auth/react";
import Image from "next/image";
import Dropdownmember from "@/public/assets/images/DropDownmember.svg"
import remove_member from "@/public/assets/images/remove-member.svg"


export type Member = {
  id?: string;
  name: string;
  email: string;
  role: string;
  date_joined: string;
};

// Actions component for the dropdown
interface MemberActionsProps {
  memberId?: string;
  memberName: string;
  onRemoveMember: (memberName: string, memberId?: string) => void;
}

function MemberActions({ memberId, memberName, onRemoveMember }: MemberActionsProps) {
    const [isOpen, setIsOpen] = React.useState(false);

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger asChild>
        <Button 
          variant="ghost" 
          className="h-8 w-8 p-0 focus:outline-none focus-visible:outline-none focus-visible:ring-0 hover:opacity-80"
        >
          <div className="flex items-center justify-center">
            <div className={`w-10 h-10 rounded-lg flex flex-col border-[2px] items-center justify-center gap-0.5 transition-colors ${
              isOpen ? 'bg-[#204ECF]' : 'bg-gray-100 hover:bg-gray-200'
            }`}>
              <div className={`w-1 h-1 rounded-full ${isOpen ? 'bg-white' : 'bg-gray-600'}`}></div>
              <div className={`w-1 h-1 rounded-full ${isOpen ? 'bg-white' : 'bg-gray-600'}`}></div>
              <div className={`w-1 h-1 rounded-full ${isOpen ? 'bg-white' : 'bg-gray-600'}`}></div>
            </div>
          </div>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="bg-white border shadow-lg rounded-lg">
        <DropdownMenuItem
          onClick={() => onRemoveMember(memberName, memberId)}
          className="text-black cursor-pointer flex items-center gap-2 px-3 py-2"
        >
          <div className="h-4 w-4">
            <Image src={remove_member} alt="Remove" />
          </div>
          Remove Member
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function MemberTable() {
  const { data: session, status } = useSession();
  console.log("Session:", session);
  console.log("Status:", status);

  const [members, setMembers] = React.useState<Member[]>([]);
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string>("");

  const [sorting, setSorting] = React.useState<SortingState>([]);
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>([]);
  const [columnVisibility, setColumnVisibility] = React.useState<VisibilityState>({});

  const [page, setPage] = React.useState(1);
  const [totalPages, setTotalPages] = React.useState(1);

  // Handle remove member
  const handleRemoveMember = async (memberName: string, memberId?: string) => {
    if (confirm(`Are you sure you want to remove ${memberName}?`)) {
      try {
        // Add your remove member API call here
        // const response = await fetch(`/api/account/users/${memberId}`, {
        //   method: 'DELETE',
        // });
        
        // For now, just remove from local state
        setMembers(prev => prev.filter(member => 
          memberId ? member.id !== memberId : member.name !== memberName
        ));
        
        console.log(`Removing member: ${memberName} (ID: ${memberId})`);
      } catch (error) {
        console.error('Failed to remove member:', error);
      }
    }
  };

  // Define columns with actions
  const columns: ColumnDef<Member>[] = [
    {
      accessorKey: "name",
      header: "Name",
      cell: ({ row }) => {
        return (
          <div className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer">
            {row.getValue("name")}
          </div>
        );
      },
    },
    {
      accessorKey: "email",
      header: "Email",
      cell: ({ row }) => {
        return (
          <div className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer">
            {row.getValue("email")}
          </div>
        );
      },
    },
    {
      accessorKey: "role",
      header: "Role",
      cell: ({ row }) => {
        return (
          <div className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer">
            {row.getValue("role")}
          </div>
        );
      },
    },
    {
      accessorKey: "date_joined",
      header: "Date Joined",
      cell: ({ row }) => {
        return (
          <div className="font-normal text-grey-dark hover:text-blue-primary hover:underline cursor-pointer">
            {row.getValue("date_joined")}
          </div>
        );
      },
    },
    {
      id: "actions",
      header: "",
      enableHiding: false,
      cell: ({ row }) => (
        <MemberActions
          memberId={row.original.id}
          memberName={row.original.name}
          onRemoveMember={handleRemoveMember}
        />
      ),
    },
  ];

  // Fetch members
  const fetchMembers = async (pageNum = 1) => {
    setIsLoading(true);
    setError("");
    try {
      const res = await fetch(`/api/account/users?page=${pageNum}&per_page=10`);
      const result = await res.json();

      if (!res.ok) {
        const errorMessage =
          typeof result.error === "string"
            ? result.error
            : result.error?.message ||
              JSON.stringify(result.error) ||
              "Failed to fetch members";
        throw new Error(errorMessage);
      }

      const apiItems = (result?.data?.items ?? []) as Array<{
        id?: string;
        username?: string;
        email?: string;
        role_access_level?: string;
        created_at?: string;
      }>;
      setTotalPages(result?.pagination?.total_pages || 1);

      if (apiItems.length === 0) {
        setMembers([]);
        setError("No members available...");
        return;
      }

      const transformed: Member[] = apiItems.map((item) => ({
        id: item.id,
        name: item.username ?? "",
        email: item.email ?? "",
        role: item.role_access_level ?? "",
        date_joined: item.created_at ? new Date(item.created_at).toLocaleString() : "",
      }));

      setMembers(transformed);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Something went wrong");
    } finally {
      setIsLoading(false);
    }
  };

  React.useEffect(() => {
    fetchMembers(page);
  }, [page]);

  const table = useReactTable({
    data: members,
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
                  <h1 className="text-2xl font-medium text-grey-dark">Members</h1>
                  <span className="bg-cerulean-blue text-grey-dark px-3 py-1 rounded-2xl text-sm font-medium">
                    {members.length}
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
                  Loading members...
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
      </div>
    </div>
  );
}