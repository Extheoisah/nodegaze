"use client";

import { useState, useEffect } from "react";
import { useRouter, useParams } from "next/navigation";
import { AppLayout } from "@/components/app-layout";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ArrowLeft, Eye, EyeOff, ChevronDown } from "lucide-react";

interface NotificationDetails {
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

interface MessageAttempt {
  id: string;
  event_id: string;
  status: "Succeeded" | "Failed";
  event_type:
    | "InvoiceCreated"
    | "InvoiceSettled"
    | "ChannelOpened"
    | "ChannelClosed";
  message_id: string;
  date: string;
}

export default function EndpointDetailsPage() {
  const router = useRouter();
  const params = useParams();
  const [notification, setNotification] = useState<NotificationDetails | null>(
    null
  );
  const [messageAttempts, setMessageAttempts] = useState<MessageAttempt[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [showSecret, setShowSecret] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const itemsPerPage = 10;

  const webhookId = params.id as string;

  // Mock data for message attempts (in a real app, this would come from an API)
  const mockMessageAttempts: MessageAttempt[] = [
    {
      id: "1",
      event_id: "01983ca4-f7df-7122-8301-590d840e5dd0",
      status: "Succeeded",
      event_type: "InvoiceSettled",
      message_id: "msg_ighwfgtuydclgjhvsydtfguv",
      date: "15th Feb, 2025 14:40",
    },
    {
      id: "2",
      event_id: "01983ca4-f7df-7122-8301-590d840e5dd1",
      status: "Failed",
      event_type: "ChannelOpened",
      message_id: "msg_abc123def456ghi789",
      date: "15th Feb, 2025 13:25",
    },
    {
      id: "3",
      event_id: "01983ca4-f7df-7122-8301-590d840e5dd2",
      status: "Succeeded",
      event_type: "InvoiceCreated",
      message_id: "msg_xyz789uvw456rst123",
      date: "15th Feb, 2025 12:10",
    },
    {
      id: "4",
      event_id: "01983ca4-f7df-7122-8301-590d840e5dd3",
      status: "Succeeded",
      event_type: "ChannelClosed",
      message_id: "msg_qwe987asd654zxc321",
      date: "15th Feb, 2025 11:55",
    },
    {
      id: "5",
      event_id: "01983ca4-f7df-7122-8301-590d840e5dd4",
      status: "Failed",
      event_type: "InvoiceSettled",
      message_id: "msg_mno765pqr432stu987",
      date: "15th Feb, 2025 10:30",
    },
  ];

  const totalPages = Math.ceil(messageAttempts.length / itemsPerPage);
  const startIndex = (currentPage - 1) * itemsPerPage;
  const currentData = messageAttempts.slice(
    startIndex,
    startIndex + itemsPerPage
  );

  const fetchNotificationDetails = async () => {
    if (!webhookId) return;

    setIsLoading(true);
    setError("");

    try {
      const response = await fetch("/api/notifications", {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      });

      const result = await response.json();

      if (!response.ok) {
        if (response.status === 401) {
          router.push("/login");
          return;
        }
        throw new Error(result.error || "Failed to fetch notification details");
      }
      console.log("Notification details result:", result);

      if (result.success && result.data) {
        // Find the notification by URL or some other identifier
        // For now, we'll use the first one as a placeholder
        const foundNotification =
          result.data.find((notif: NotificationDetails) =>
            notif.url.includes(decodeURIComponent(webhookId))
          ) || result.data[0];

        if (foundNotification) {
          setNotification(foundNotification);
          // In a real app, you would fetch message attempts for this specific notification
          setMessageAttempts(mockMessageAttempts);
        } else {
          setError("Notification not found");
        }
      } else {
        setError("Failed to load notification details");
      }
    } catch (error) {
      console.error("Failed to fetch notification details:", error);
      setError(
        error instanceof Error
          ? error.message
          : "Failed to load notification details"
      );
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchNotificationDetails();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [webhookId, router]);

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString("en-US", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const formatEventType = (eventType: string) => {
    // Convert camelCase to Title Case with spaces
    return eventType.replace(/([A-Z])/g, " $1").trim();
  };

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

  if (isLoading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-64">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
          <span className="ml-2 text-grey-accent">
            Loading endpoint details...
          </span>
        </div>
      </AppLayout>
    );
  }

  if (error || !notification) {
    return (
      <AppLayout>
        <div className="space-y-6">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              onClick={() => router.push("/events")}
              className="flex items-center gap-2"
            >
              <ArrowLeft className="h-4 w-4" />
              Back
            </Button>
          </div>
          <div className="text-center py-16">
            <p className="text-red-500 text-lg">
              {error || "Notification not found"}
            </p>
          </div>
        </div>
      </AppLayout>
    );
  }

  return (
    <AppLayout>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            onClick={() => router.push("/events")}
            className="flex items-center gap-2"
          >
            <ArrowLeft className="h-4 w-4" />
            Back
          </Button>
          <div className="text-sm text-muted-foreground">
            Events &gt; Event Details
          </div>
        </div>

        <h1 className="text-3xl font-bold text-grey-dark">Endpoint Details</h1>

        {/* Notification Details */}
        <div className="bg-white rounded-xl border">
          <div className="p-6 border-b">
            <h2 className="text-lg font-medium text-grey-dark">
              {notification.notification_type} Details
            </h2>
          </div>
          <div className="p-6">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <div className="space-y-6">
                <div>
                  <label className="text-sm text-grey-accent">URL</label>
                  <div className="flex items-center gap-2 mt-1">
                    <p className="text-sm text-grey-dark font-mono break-all">
                      {notification.url}
                    </p>
                  </div>
                </div>
                <div>
                  <label className="text-sm text-grey-accent">
                    Description
                  </label>
                  <p className="text-sm text-grey-dark mt-1">
                    {notification.name}
                  </p>
                </div>
                {notification.notification_type === "Webhook" && (
                  <div>
                    <label className="text-sm text-grey-accent">Secret</label>
                    <div className="flex items-center gap-2 mt-1">
                      <p className="text-sm text-grey-dark font-mono">
                        {showSecret ? "your-webhook-secret" : "••••••••"}
                      </p>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setShowSecret(!showSecret)}
                      >
                        {showSecret ? (
                          <EyeOff className="h-4 w-4" />
                        ) : (
                          <Eye className="h-4 w-4" />
                        )}
                      </Button>
                    </div>
                  </div>
                )}
                <div>
                  <label className="text-sm text-grey-accent">Fail</label>
                  <div className="flex items-center gap-2 mt-1">
                    <span className="text-lg font-semibold text-grey-dark">
                      190
                    </span>
                    <span className="text-sm text-red-500">27.7%</span>
                  </div>
                </div>
              </div>
              <div className="space-y-6">
                <div>
                  <label className="text-sm text-grey-accent">
                    Creation Date
                  </label>
                  <p className="text-sm text-grey-dark mt-1">
                    {formatDate(notification.created_at)}
                  </p>
                </div>
                <div>
                  <label className="text-sm text-grey-accent">
                    Last Updated
                  </label>
                  <p className="text-sm text-grey-dark mt-1">
                    {formatDate(notification.updated_at)}
                  </p>
                </div>
                <div>
                  <label className="text-sm text-grey-accent">Success</label>
                  <div className="flex items-center gap-2 mt-1">
                    <span className="text-lg font-semibold text-grey-dark">
                      500
                    </span>
                    <span className="text-sm text-success-green">72.3%</span>
                  </div>
                </div>
                <div>
                  <label className="text-sm text-grey-accent">Sending</label>
                  <p className="text-lg font-semibold text-grey-dark mt-1">5</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Status Tabs */}
        <div className="flex gap-4">
          <Button
            variant="default"
            size="sm"
            className="bg-blue-primary text-white"
          >
            All{" "}
            <span className="ml-1 bg-white text-blue-primary px-2 py-0.5 rounded-full text-xs">
              {messageAttempts.length}
            </span>
          </Button>
          <Button variant="outline" size="sm">
            Succeeded{" "}
            <span className="ml-1 bg-gray-100 px-2 py-0.5 rounded-full text-xs">
              {messageAttempts.filter((m) => m.status === "Succeeded").length}
            </span>
          </Button>
          <Button variant="outline" size="sm">
            Failed{" "}
            <span className="ml-1 bg-gray-100 px-2 py-0.5 rounded-full text-xs">
              {messageAttempts.filter((m) => m.status === "Failed").length}
            </span>
          </Button>
        </div>

        {/* Message Attempts */}
        <div className="bg-white rounded-xl border">
          <div className="p-6 border-b">
            <h2 className="text-lg font-medium text-grey-dark">
              Message Attempts{" "}
              <span className="text-sm font-normal text-grey-accent">
                {messageAttempts.length}
              </span>
            </h2>
          </div>
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Status
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Event Type
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Message ID
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Date
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {currentData.length === 0 ? (
                  <TableRow>
                    <TableCell
                      colSpan={5}
                      className="px-6 py-8 text-center text-grey-accent"
                    >
                      No message attempts found.
                    </TableCell>
                  </TableRow>
                ) : (
                  currentData.map((attempt) => (
                    <TableRow
                      key={attempt.id}
                      className="cursor-pointer hover:bg-gray-50"
                      onClick={() =>
                        router.push(
                          `/events/${webhookId}/message?eventId=${attempt.event_id}`
                        )
                      }
                    >
                      <TableCell className="px-6 py-4">
                        <span
                          className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                            attempt.status === "Succeeded"
                              ? "bg-green-100 text-green-800"
                              : "bg-red-100 text-red-800"
                          }`}
                        >
                          {attempt.status}
                        </span>
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark">
                        {formatEventType(attempt.event_type)}
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark font-mono">
                        {attempt.message_id}
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark">
                        {attempt.date}
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
          {totalPages > 1 && (
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
              </div>
            </div>
          )}
        </div>
      </div>
    </AppLayout>
  );
}
