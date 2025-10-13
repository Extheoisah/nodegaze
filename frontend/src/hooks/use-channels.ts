import { useQuery } from "@tanstack/react-query";
import { keepPreviousData } from "@tanstack/react-query";

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

/**
 * Transform API channel data to display format
 */
function transformChannel(item: ApiChannel): Channel {
  const rawState = (item.channel_state ?? "").toString().toLowerCase();
  const state: Channel["state"] =
    rawState === "active" || rawState === "inactive" || rawState === "pending"
      ? (rawState as Channel["state"])
      : "Disabled";

  return {
    id: item.chan_id,
    channel_name:
      item.alias && item.alias.trim() !== ""
        ? item.alias
        : String(item.chan_id),
    state,
    inbound_balance: Number(item.remote_balance ?? 0),
    outbound_balance: Number(item.local_balance ?? 0),
    last_updated: new Date((item.last_update ?? 0) * 1000).toLocaleString(),
    uptime: item.uptime,
  };
}

/**
 * Fetches paginated channels for table display
 * Shows stale data while refetching to avoid flash of loading state
 */
export function useChannels(page: number, perPage: number = 10) {
  return useQuery({
    queryKey: ["channels", { page, perPage }],
    queryFn: async () => {
      const res = await fetch(`/api/channels?page=${page}&per_page=${perPage}`);
      const result = await res.json();

      if (!res.ok) {
        throw new Error(
          typeof result.error === "string"
            ? result.error
            : result.error?.message ||
              JSON.stringify(result.error) ||
              "No Channels Available..."
        );
      }

      const apiItems = (result?.data?.items ?? []) as ApiChannel[];
      const totalPages = result?.data?.total_pages || 1;

      if (apiItems.length === 0) {
        return {
          channels: [],
          totalPages: 1,
          error: "No channels available...",
        };
      }

      const transformed: Channel[] = apiItems.map(transformChannel);

      return {
        channels: transformed,
        totalPages,
        error: null,
      };
    },
    staleTime: 30000, // 30 seconds
    placeholderData: keepPreviousData, // Show stale data while fetching
  });
}
