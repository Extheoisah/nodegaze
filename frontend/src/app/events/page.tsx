"use client";

import { useState } from "react";
import { AppLayout } from "@/components/app-layout";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ChevronDown, ChevronUp, Plus } from "lucide-react";
import { NotificationDialog } from "@/components/notification-dialog";

const endpointsData = [
  {
    id: 1,
    url: "https://api.your-application.com/webhooks/receive_data_12345",
    successRate: 0,
    failRate: 0,
  },
  // Add more mock data for pagination
  ...Array.from({ length: 29 }, (_, i) => ({
    id: i + 2,
    url: `https://api.example${i + 2}.com/webhooks/endpoint_${i + 2}`,
    successRate: Math.floor(Math.random() * 100),
    failRate: Math.floor(Math.random() * 100),
  })),
];

export default function EventsPage() {
  const [isNotificationSettingsOpen, setIsNotificationSettingsOpen] =
    useState(true);
  const [isCategoriesOpen, setIsCategoriesOpen] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedEvent, setSelectedEvent] = useState<string>("");
  const itemsPerPage = 1;

  const totalPages = Math.ceil(endpointsData.length / itemsPerPage);
  const startIndex = (currentPage - 1) * itemsPerPage;
  const currentData = endpointsData.slice(
    startIndex,
    startIndex + itemsPerPage
  );

  const getPaginationNumbers = () => {
    const pages = [];
    const maxVisible = 5;
    let start = Math.max(1, currentPage - Math.floor(maxVisible / 2));
    const end = Math.min(totalPages, start + maxVisible - 1);

    if (end - start + 1 < maxVisible) {
      start = Math.max(1, end - maxVisible + 1);
    }

    for (let i = start; i <= end; i++) {
      pages.push(i);
    }
    return pages;
  };

  const handleSubmit = (data: {
    eventType: string;
    url: string;
    secret?: string;
    description: string;
  }) => {
    // Handle form submission here
    console.log(data);
  };

  return (
    <AppLayout>
      <div className="space-y-6">
        {/* Notification Settings Section */}
        <div className="bg-white rounded-xl border">
          <div
            className="flex items-center justify-between p-6 cursor-pointer"
            onClick={() =>
              setIsNotificationSettingsOpen(!isNotificationSettingsOpen)
            }
          >
            <h2 className="text-lg font-medium text-grey-dark">
              Notification Settings
            </h2>
            {isNotificationSettingsOpen ? (
              <ChevronUp className="h-5 w-5 text-grey-accent" />
            ) : (
              <ChevronDown className="h-5 w-5 text-grey-accent" />
            )}
          </div>

          {isNotificationSettingsOpen && (
            <div className="px-6 pb-6 border-t">
              <div className="pt-6 space-y-4">
                <p className="text-grey-accent">
                  Configure notification events
                </p>
                <div className="flex items-center gap-4">
                  <Select
                    value={selectedEvent}
                    onValueChange={setSelectedEvent}
                  >
                    <SelectTrigger className="w-full max-w-md text-grey-dark font-medium py-6">
                      <SelectValue placeholder="Select" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="webhook">Webhook</SelectItem>
                      <SelectItem value="discord">Discord</SelectItem>
                    </SelectContent>
                  </Select>
                  <NotificationDialog
                    selectedEvent={selectedEvent}
                    onSubmit={handleSubmit}
                  />
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Endpoints Section */}
        <div className="bg-white rounded-xl border">
          <div className="p-6 border-b">
            <div className="flex items-center gap-3">
              <h2 className="text-lg font-medium text-grey-dark">Endpoints</h2>
              <span className="bg-cerulean-blue text-grey-dark px-2 py-1 rounded-xl text-sm font-medium">
                {endpointsData.length}
              </span>
            </div>
          </div>

          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    URL
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Success Rate
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Fail Rate
                  </TableHead>
                  <TableHead className="w-12"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {currentData.map((endpoint) => (
                  <TableRow key={endpoint.id}>
                    <TableCell className="px-6 py-4 text-sm text-grey-dark font-mono">
                      {endpoint.url}
                    </TableCell>
                    <TableCell className="px-6 py-4 text-sm">
                      <span
                        className={`font-medium ${
                          endpoint.successRate === 0
                            ? "text-red-500"
                            : "text-success-green"
                        }`}
                      >
                        {endpoint.successRate}%
                      </span>
                    </TableCell>
                    <TableCell className="px-6 py-4 text-sm">
                      <span
                        className={`font-medium ${
                          endpoint.failRate === 0
                            ? "text-red-500"
                            : "text-success-green"
                        }`}
                      >
                        {endpoint.failRate}%
                      </span>
                    </TableCell>
                    <TableCell className="px-6 py-4">
                      <Button variant="ghost" className="h-8 w-8 p-0">
                        <ChevronDown className="h-4 w-4 rotate-[-90deg]" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>

          {/* Pagination */}
          <div className="flex items-center justify-between px-6 py-4 border-t">
            <div className="text-sm text-grey-accent">
              Page {currentPage} of {totalPages}
            </div>
            <div className="flex items-center gap-2">
              {getPaginationNumbers().map((pageNum) => (
                <Button
                  key={pageNum}
                  variant={currentPage === pageNum ? "default" : "outline"}
                  size="sm"
                  onClick={() => setCurrentPage(pageNum)}
                  className={`w-8 h-8 p-0 ${
                    currentPage === pageNum
                      ? "bg-blue-primary text-white"
                      : "text-grey-accent hover:text-grey-dark"
                  }`}
                >
                  {pageNum}
                </Button>
              ))}
              <div className="flex items-center gap-2 ml-4">
                <span className="text-sm text-grey-accent">Go to page</span>
                <Select
                  value={currentPage.toString()}
                  onValueChange={(value: string) =>
                    setCurrentPage(parseInt(value))
                  }
                >
                  <SelectTrigger className="w-16 h-8">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Array.from({ length: totalPages }, (_, i) => (
                      <SelectItem key={i + 1} value={(i + 1).toString()}>
                        {i + 1}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
        </div>

        {/* Categories Section */}
        <div className="bg-white rounded-xl border">
          <div
            className="flex items-center justify-between p-6 cursor-pointer"
            onClick={() => setIsCategoriesOpen(!isCategoriesOpen)}
          >
            <h2 className="text-lg font-medium text-grey-dark">Categories</h2>
            <Plus className="h-5 w-5 text-grey-accent" />
          </div>

          {isCategoriesOpen && (
            <div className="px-6 pb-6 border-t">
              <div className="pt-6">
                <p className="text-grey-accent">
                  Configure notification categories and settings here.
                </p>
                {/* Add categories content here when needed */}
              </div>
            </div>
          )}
        </div>
      </div>
    </AppLayout>
  );
}
