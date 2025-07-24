"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
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

interface Notification {
  id: string;
  account_id: string;
  user_id: string;
  name: string;
  notification_type: "Webhook" | "Discord";
  url: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
  is_deleted: boolean;
  deleted_at: string | null;
}

export default function EventsPage() {
  const router = useRouter();
  const [isNotificationSettingsOpen, setIsNotificationSettingsOpen] =
    useState(true);
  const [isCategoriesOpen, setIsCategoriesOpen] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedEvent, setSelectedEvent] = useState<string>("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [isLoadingNotifications, setIsLoadingNotifications] = useState(false);
  const [notificationsError, setNotificationsError] = useState<string>("");
  const itemsPerPage = 5;

  const totalPages = Math.ceil(notifications.length / itemsPerPage);
  const startIndex = (currentPage - 1) * itemsPerPage;
  const currentData = notifications.slice(
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

  const fetchNotifications = async () => {
    if (status === "unauthenticated") {
      router.push("/login");
      return;
    }

    if (status === "loading") {
      return; // Wait for session to load
    }

    setIsLoadingNotifications(true);
    setNotificationsError("");

    try {
      const response = await fetch("/api/notifications", {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      });

      const result = await response.json();

      if (!response.ok) {
        // If unauthorized, redirect to login
        if (response.status === 401) {
          router.push("/login");
          return;
        }
        throw new Error(result.error || "Failed to fetch notifications");
      }

      if (result.success && result.data) {
        setNotifications(result.data);
      } else {
        setNotifications([]);
      }
    } catch (error) {
      console.error("Failed to fetch notifications:", error);
      setNotificationsError(
        error instanceof Error ? error.message : "Failed to load notifications"
      );
    } finally {
      setIsLoadingNotifications(false);
    }
  };

  useEffect(() => {
    fetchNotifications();
    // we don't need to include fetchNotifications in the dependency array because of a circular dependency
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [router]);

  const handleSubmit = async (data: {
    eventType: string;
    url: string;
    secret?: string;
    description: string;
  }) => {
    // Check if user is authenticated
    if (status === "unauthenticated") {
      router.push("/login");
      return;
    }

    if (status === "loading") {
      return; // Wait for session to load
    }

    setIsSubmitting(true);

    try {
      // Map eventType to the expected notification_type format
      const notification_type =
        data.eventType === "webhook" ? "Webhook" : "Discord";

      const response = await fetch("/api/notifications", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          name: data.description || `${notification_type} Alert`,
          notification_type: notification_type,
          url: data.url,
        }),
      });

      const result = await response.json();

      if (!response.ok) {
        // If unauthorized, redirect to login
        if (response.status === 401) {
          router.push("/login");
          return;
        }
        throw new Error(
          result.error ||
            `Failed to create ${notification_type.toLowerCase()} notification`
        );
      }

      // Show success message
      alert(
        result.message ||
          `${notification_type} notification created successfully!`
      );

      // Reset the selected event
      setSelectedEvent("");

      // Refresh the notifications list
      fetchNotifications();

      console.log("Notification created:", result.data);
    } catch (error) {
      console.error("Failed to create notification:", error);
      alert(
        error instanceof Error
          ? error.message
          : "Failed to create notification. Please try again."
      );
    } finally {
      setIsSubmitting(false);
    }
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
                    isLoading={isSubmitting}
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
                {notifications.length}
              </span>
            </div>
          </div>

          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Name
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Type
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    URL
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Status
                  </TableHead>
                  <TableHead className="w-12"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoadingNotifications ? (
                  <TableRow>
                    <TableCell colSpan={5} className="px-6 py-8 text-center">
                      <div className="flex items-center justify-center">
                        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900"></div>
                        <span className="ml-2 text-grey-accent">
                          Loading notifications...
                        </span>
                      </div>
                    </TableCell>
                  </TableRow>
                ) : notificationsError ? (
                  <TableRow>
                    <TableCell
                      colSpan={5}
                      className="px-6 py-8 text-center text-red-500"
                    >
                      {notificationsError}
                    </TableCell>
                  </TableRow>
                ) : currentData.length === 0 ? (
                  <TableRow>
                    <TableCell
                      colSpan={5}
                      className="px-6 py-8 text-center text-grey-accent"
                    >
                      No notifications configured yet.
                    </TableCell>
                  </TableRow>
                ) : (
                  currentData.map((notification) => (
                    <TableRow key={notification.id}>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark font-medium">
                        {notification.name}
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm">
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                          {notification.notification_type}
                        </span>
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark font-mono break-all">
                        {notification.url}
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm">
                        <span
                          className={`font-medium ${
                            notification.is_active
                              ? "text-success-green"
                              : "text-red-500"
                          }`}
                        >
                          {notification.is_active ? "Active" : "Inactive"}
                        </span>
                      </TableCell>
                      <TableCell className="px-6 py-4">
                        <Button variant="ghost" className="h-8 w-8 p-0">
                          <ChevronDown className="h-4 w-4 rotate-[-90deg]" />
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))
                )}
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
