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
import { ArrowLeft, Eye, EyeOff } from "lucide-react";
import Link from "next/link";

interface NotificationDetails {
  id: string;
  account_id: string;
  user_id: string;
  name: string;
  notification_type: "Webhook" | "Discord";
  url: string;
  secret?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
  is_deleted: boolean;
  deleted_at: string | null;
}

interface EventData {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
}

interface Event {
  id: string;
  account_id: string;
  user_id: string;
  node_id: string;
  node_alias: string;
  event_type:
    | "InvoiceCreated"
    | "InvoiceSettled"
    | "ChannelOpened"
    | "ChannelClosed";
  severity: "Info" | "Warning" | "Error";
  title: string;
  description: string;
  notifications_id: string;
  data: EventData;
  timestamp: string;
  created_at: string;
}

interface EventsResponse {
  success: boolean;
  data: {
    items: Event[];
    total: number;
  };
  message: string;
  timestamp: string;
}

export default function EndpointDetailsPage() {
  const router = useRouter();
  const params = useParams();
  const [notification, setNotification] = useState<NotificationDetails | null>(
    null
  );
  const [events, setEvents] = useState<Event[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [showSecret, setShowSecret] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [severityFilter, setSeverityFilter] = useState<string>("all");
  const itemsPerPage = 10;

  const notificationId = params.id as string;

  // Filter events based on severity filter
  const filteredEvents = events.filter((event) => {
    if (severityFilter === "all") return true;
    return event.severity === severityFilter;
  });

  const totalPages = Math.ceil(filteredEvents.length / itemsPerPage);
  const startIndex = (currentPage - 1) * itemsPerPage;
  const currentData = filteredEvents.slice(
    startIndex,
    startIndex + itemsPerPage
  );

  const fetchNotificationAndEvents = async () => {
    if (!notificationId) return;

    setIsLoading(true);
    setError("");

    try {
      // Fetch notifications and events in parallel
      const [notificationsResponse, eventsResponse] = await Promise.all([
        fetch("/api/notifications", {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
          },
        }),
        fetch("/api/events", {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
          },
        }),
      ]);

      // Handle notifications response
      if (!notificationsResponse.ok) {
        if (notificationsResponse.status === 401) {
          router.push("/login");
          return;
        }
        throw new Error("Failed to fetch notification details");
      }

      // Handle events response
      if (!eventsResponse.ok) {
        if (eventsResponse.status === 401) {
          router.push("/login");
          return;
        }
        throw new Error("Failed to fetch events");
      }

      const notificationsResult = await notificationsResponse.json();
      const eventsResult: EventsResponse = await eventsResponse.json();

      console.log("Notifications result:", notificationsResult);
      console.log("Events result:", eventsResult);

      // Find the notification by ID
      if (notificationsResult.success && notificationsResult.data) {
        const foundNotification = notificationsResult.data.find(
          (notif: NotificationDetails) => notif.id === notificationId
        );

        if (foundNotification) {
          setNotification(foundNotification);
        } else {
          setError("Notification not found");
          return;
        }
      } else {
        setError("Failed to load notification details");
        return;
      }

      // Filter events by notification ID
      if (eventsResult.success && eventsResult.data) {
        const filteredEvents = eventsResult.data.items.filter(
          (event: Event) => event.notifications_id === notificationId
        );
        setEvents(filteredEvents);
      } else {
        setError("Failed to load events");
      }
    } catch (error) {
      console.error("Failed to fetch data:", error);
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
    fetchNotificationAndEvents();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [notificationId, router]);

  // Reset page when filter changes
  useEffect(() => {
    setCurrentPage(1);
  }, [severityFilter]);

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

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case "Info":
        return "bg-blue-100 text-blue-800";
      case "Warning":
        return "bg-yellow-100 text-yellow-800";
      case "Error":
        return "bg-red-100 text-red-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
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
            <Link
              href="/events"
              className="flex items-center gap-2 p-2 rounded-md border border-grey-dark/15 bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
            >
              <ArrowLeft className="h-4 w-4" />
              Back
            </Link>
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
          <Link
            href="/events"
            className="flex items-center gap-2 p-2 rounded-md border border-grey-dark/15 bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
          >
            <ArrowLeft className="h-4 w-4" />
            Back
          </Link>
          <div className="text-sm text-muted-foreground">
            <span className="text-grey-accent">
              <Link href="/events">Events</Link>
            </span>{" "}
            <span className="text-grey-accent">&gt;</span>{" "}
            <span className="text-blue-primary">{notification?.id}</span>
          </div>
        </div>

        <h1 className="text-3xl font-bold text-grey-dark">
          Notification Events
        </h1>

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
                        {showSecret
                          ? notification?.secret || "null"
                          : "••••••••"}
                      </p>
                      <Button
                        size="sm"
                        className="bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
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
                {/* <div>
                  <label className="text-sm text-grey-accent">Fail</label>
                  <div className="flex items-center gap-2 mt-1">
                    <span className="text-lg font-semibold text-grey-dark">
                      190
                    </span>
                    <span className="text-sm text-red-500">27.7%</span>
                  </div>
                </div> */}
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
                      {events.filter((e) => e.severity === "Info").length}
                    </span>
                    <span className="text-sm text-success-green">
                      {events.length > 0
                        ? (
                            (events.filter((e) => e.severity === "Info")
                              .length /
                              events.length) *
                            100
                          ).toFixed(1)
                        : "0"}
                      %
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Status Tabs */}
        <div className="flex gap-4">
          <Button
            variant={severityFilter === "all" ? "default" : "outline"}
            size="lg"
            className={
              severityFilter === "all"
                ? "bg-blue-primary text-white hover:bg-blue-primary"
                : "bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
            }
            onClick={() => setSeverityFilter("all")}
          >
            All Events{" "}
            <span
              className={`ml-1 px-2 py-0.5 rounded-full text-xs ${
                severityFilter === "all"
                  ? "bg-white text-blue-primary"
                  : "bg-gray-100"
              }`}
            >
              {events.length}
            </span>
          </Button>
          <Button
            variant={severityFilter === "Info" ? "default" : "outline"}
            size="lg"
            className={
              severityFilter === "Info"
                ? "bg-blue-primary text-white hover:bg-blue-primary"
                : "bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
            }
            onClick={() => setSeverityFilter("Info")}
          >
            Info{" "}
            <span
              className={`ml-1 px-2 py-0.5 rounded-full text-xs ${
                severityFilter === "Info"
                  ? "bg-white text-blue-primary"
                  : "bg-gray-100"
              }`}
            >
              {events.filter((e) => e.severity === "Info").length}
            </span>
          </Button>
          <Button
            variant={severityFilter === "Warning" ? "default" : "outline"}
            size="lg"
            className={
              severityFilter === "Warning"
                ? "bg-blue-primary text-white hover:bg-blue-primary"
                : "bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
            }
            onClick={() => setSeverityFilter("Warning")}
          >
            Warning{" "}
            <span
              className={`ml-1 px-2 py-0.5 rounded-full text-xs ${
                severityFilter === "Warning"
                  ? "bg-white text-blue-primary"
                  : "bg-gray-100"
              }`}
            >
              {events.filter((e) => e.severity === "Warning").length}
            </span>
          </Button>
          <Button
            variant={severityFilter === "Error" ? "default" : "outline"}
            size="lg"
            className={
              severityFilter === "Error"
                ? "bg-blue-primary text-white hover:bg-blue-primary"
                : "bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
            }
            onClick={() => setSeverityFilter("Error")}
          >
            Error{" "}
            <span
              className={`ml-1 px-2 py-0.5 rounded-full text-xs ${
                severityFilter === "Error"
                  ? "bg-white text-blue-primary"
                  : "bg-gray-100"
              }`}
            >
              {events.filter((e) => e.severity === "Error").length}
            </span>
          </Button>
        </div>

        {/* Events */}
        <div className="bg-white rounded-xl border">
          <div className="p-6 border-b">
            <h2 className="text-lg font-medium text-grey-dark">Events</h2>
          </div>
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Severity
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Event Type
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Description
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Node
                  </TableHead>
                  <TableHead className="text-grey-table-header font-medium text-sm py-3 px-6">
                    Timestamp
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {currentData.length === 0 ? (
                  <TableRow>
                    <TableCell
                      colSpan={5}
                      className="px-6 py-8 text-center text-grey-accent"
                    >
                      No events found for this notification.
                    </TableCell>
                  </TableRow>
                ) : (
                  currentData.map((event) => (
                    <TableRow
                      key={event.id}
                      className="cursor-pointer hover:bg-gray-50"
                      onClick={() =>
                        router.push(
                          `/events/${notificationId}/message?eventId=${event.id}`
                        )
                      }
                    >
                      <TableCell className="px-6 py-4">
                        <span
                          className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${getSeverityColor(
                            event.severity
                          )}`}
                        >
                          {event.severity}
                        </span>
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark">
                        {formatEventType(event.event_type)}
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark max-w-md truncate">
                        {event.description}
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark">
                        <div>
                          <div className="font-medium">{event.node_alias}</div>
                          <div className="text-xs text-grey-accent font-mono">
                            {event.node_id.substring(0, 16)}...
                          </div>
                        </div>
                      </TableCell>
                      <TableCell className="px-6 py-4 text-sm text-grey-dark">
                        {formatDate(event.timestamp)}
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
                Page {currentPage} of {totalPages} • Showing{" "}
                {currentData.length} of {filteredEvents.length} events
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
