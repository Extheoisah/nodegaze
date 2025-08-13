"use client";

import { useState, useEffect } from "react";
import { useRouter, useSearchParams, useParams } from "next/navigation";
import { AppLayout } from "@/components/app-layout";
import { Button } from "@/components/ui/button";
import { ArrowLeft } from "lucide-react";

interface EventData {
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
  severity: string;
  title: string;
  description: string;
  data: Record<string, unknown>;
  timestamp: string;
  created_at: string;
}

export default function MessageInformationPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const params = useParams();
  const [eventData, setEventData] = useState<EventData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [showRaw, setShowRaw] = useState(false);

  // Get eventId from URL search parameters and notificationId from route params
  const eventId = searchParams.get("eventId");
  const notificationId = params.id as string;

  const fetchEventData = async () => {
    if (!eventId) {
      setError("No event ID provided");
      setIsLoading(false);
      return;
    }

    setIsLoading(true);
    setError("");

    try {
      const response = await fetch(`/api/events/${eventId}`, {
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
        throw new Error(result.error || "Failed to fetch event details");
      }

      if (result.success && result.data) {
        setEventData(result.data);
      } else {
        setError("Failed to load event details");
      }
    } catch (error) {
      console.error("Failed to fetch event details:", error);
      setError(
        error instanceof Error ? error.message : "Failed to load event details"
      );
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchEventData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [eventId, router]);

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString("en-GB", {
      day: "2-digit",
      month: "long",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const formatEventTypeDisplay = (eventType: string) => {
    // Just return the original event type
    return eventType;
  };

  const generateBreadcrumb = (eventData: EventData) => {
    const shortNodeAlias = eventData.node_alias?.substring(0, 6) || "unknown";
    const shortEventId = eventData.id.substring(0, 6);
    return `Endpoints > ${shortNodeAlias} > Messages > ${shortEventId}`;
  };

  const formatMessageContent = (eventData: EventData) => {
    // Create a structured message content similar to the reference image
    const messageContent = {
      accountId: eventData.account_id,
      nodeAlias: eventData.node_alias,
      eventType: formatEventTypeDisplay(eventData.event_type),
      details: eventData.data,
      nodeId: eventData.node_id,
    };

    return messageContent;
  };

  const handleBackNavigation = () => {
    router.push(`/events/${notificationId}`);
  };

  if (isLoading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-64">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
          <span className="ml-2 text-grey-accent">
            Loading message information...
          </span>
        </div>
      </AppLayout>
    );
  }

  if (error || !eventData) {
    return (
      <AppLayout>
        <div className="space-y-6">
          <div className="flex items-center gap-4">
            <Button
              variant="outline"
              size="lg"
              onClick={handleBackNavigation}
              className="flex items-center gap-2 bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
            >
              <ArrowLeft className="h-4 w-4" />
              Back
            </Button>
          </div>
          <div className="text-center py-16">
            <p className="text-red-500 text-lg">{error || "Event not found"}</p>
          </div>
        </div>
      </AppLayout>
    );
  }

  return (
    <AppLayout>
      <div className="space-y-6 max-w-full overflow-hidden">
        {/* Breadcrumb */}
        <div className="text-sm text-grey-accent">
          {generateBreadcrumb(eventData)}
        </div>

        {/* Header */}
        <div className="flex items-center gap-4">
          <Button
            variant="outline"
            size="lg"
            onClick={handleBackNavigation}
            className="flex items-center gap-2 bg-transparent text-grey-dark hover:text-grey-dark hover:bg-grey-sub-background cursor-pointer"
          >
            <ArrowLeft className="h-4 w-4" />
            Back
          </Button>
        </div>

        {/* Event Title and Date */}
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-semibold text-grey-dark">
            {formatEventTypeDisplay(eventData.event_type)}
          </h1>
          <div className="text-right">
            <p className="text-sm text-grey-accent">Created at</p>
            <p className="text-sm text-grey-dark font-medium">
              {formatDate(eventData.created_at)}
            </p>
          </div>
        </div>

        {/* Message Content */}
        <div className="bg-white rounded-xl border max-w-full overflow-hidden">
          <div className="p-6 border-b">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-medium text-grey-dark">
                Message content
              </h2>
              <div className="flex items-center gap-2">
                <span className="text-sm text-grey-accent">Raw</span>
                <button
                  onClick={() => setShowRaw(!showRaw)}
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    showRaw ? "bg-blue-primary" : "bg-gray-200"
                  }`}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                      showRaw ? "translate-x-6" : "translate-x-1"
                    }`}
                  />
                </button>
              </div>
            </div>
          </div>
          <div className="p-6">
            {showRaw ? (
              <div className="bg-gray-50 rounded-lg p-4 overflow-hidden">
                <pre className="text-sm text-grey-dark whitespace-pre-wrap break-words break-all max-w-full overflow-hidden">
                  {JSON.stringify(formatMessageContent(eventData), null, 2)}
                </pre>
              </div>
            ) : (
              <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="min-w-0">
                    <label className="text-sm font-medium text-grey-accent">
                      Account ID
                    </label>
                    <p className="text-sm text-grey-dark font-mono mt-1 break-all overflow-hidden">
                      {eventData.account_id}
                    </p>
                  </div>
                  <div>
                    <label className="text-sm font-medium text-grey-accent">
                      Event Type
                    </label>
                    <p className="text-sm text-grey-dark mt-1">
                      {formatEventTypeDisplay(eventData.event_type)}
                    </p>
                  </div>
                  <div>
                    <label className="text-sm font-medium text-grey-accent">
                      Node Alias
                    </label>
                    <p className="text-sm text-grey-dark mt-1">
                      {eventData.node_alias}
                    </p>
                  </div>
                  <div>
                    <label className="text-sm font-medium text-grey-accent">
                      Severity
                    </label>
                    <p className="text-sm text-grey-dark mt-1">
                      {eventData.severity}
                    </p>
                  </div>
                </div>

                <div>
                  <label className="text-sm font-medium text-grey-accent">
                    Description
                  </label>
                  <p className="text-sm text-grey-dark mt-1">
                    {eventData.description}
                  </p>
                </div>

                {/* Transaction/Data Details */}
                {eventData.data && Object.keys(eventData.data).length > 0 && (
                  <div>
                    <label className="text-sm font-medium text-grey-accent">
                      Details
                    </label>
                    <div className="mt-2 bg-gray-50 rounded-lg p-4 overflow-hidden">
                      <pre className="text-sm text-grey-dark whitespace-pre-wrap break-words break-all max-w-full overflow-hidden">
                        {JSON.stringify(eventData.data, null, 2)}
                      </pre>
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Event Metadata */}
        <div className="bg-white rounded-xl border max-w-full overflow-hidden">
          <div className="p-6 border-b">
            <h3 className="text-lg font-medium text-grey-dark">
              Event Metadata
            </h3>
          </div>
          <div className="p-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="min-w-0">
                <label className="text-sm font-medium text-grey-accent">
                  Event ID
                </label>
                <p className="text-sm text-grey-dark font-mono mt-1 break-all overflow-hidden">
                  {eventData.id}
                </p>
              </div>
              <div className="min-w-0">
                <label className="text-sm font-medium text-grey-accent">
                  User ID
                </label>
                <p className="text-sm text-grey-dark font-mono mt-1 break-all overflow-hidden">
                  {eventData.user_id}
                </p>
              </div>
              <div className="min-w-0">
                <label className="text-sm font-medium text-grey-accent">
                  Node ID
                </label>
                <p className="text-sm text-grey-dark font-mono mt-1 break-all overflow-hidden">
                  {eventData.node_id}
                </p>
              </div>
              <div>
                <label className="text-sm font-medium text-grey-accent">
                  Timestamp
                </label>
                <p className="text-sm text-grey-dark mt-1">
                  {formatDate(eventData.timestamp)}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </AppLayout>
  );
}
