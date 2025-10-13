import { useQuery } from "@tanstack/react-query";
import { keepPreviousData } from "@tanstack/react-query";

export type Payment = {
  state: string;
  payment_type: string;
  amount_sat: number;
  amount_usd: number;
  routing_fee: number;
  creation_time:
    | {
        secs_since_epoch: number;
        nanos_since_epoch: number;
      }
    | number;
  invoice: string;
  payment_hash: string;
  completed_at: number;
};

export type PaymentFilters = {
  paymentState?: "settled" | "failed" | "inflight";
  operator?: "gte" | "lte" | "eq";
  value?: number;
  from?: string;
  to?: string;
};

export type PaymentMetrics = {
  incomingCount: number;
  outgoingCount: number;
  totalIncomingAmount: number;
  totalOutgoingAmount: number;
  allCount: number;
};

/**
 * Fetches and calculates payment metrics
 * Shows stale data while refetching to avoid flash of zeros
 */
export function usePaymentMetrics(filters?: PaymentFilters) {
  return useQuery({
    queryKey: ["payments", "metrics", filters],
    queryFn: async (): Promise<PaymentMetrics> => {
      const params = new URLSearchParams();
      params.set("per_page", "100");
      params.set("page", "1");

      if (filters?.paymentState) {
        params.set("states", filters.paymentState);
      }
      if (filters?.operator) params.set("operator", filters.operator);
      if (typeof filters?.value === "number")
        params.set("value", String(filters.value));
      if (filters?.from) params.set("from", filters.from);
      if (filters?.to) params.set("to", filters.to);

      const res = await fetch(`/api/payments?${params.toString()}`);
      if (!res.ok) {
        throw new Error(`Failed to fetch payments: ${res.status}`);
      }

      const data = await res.json();
      const allPayments: Payment[] = data?.data?.items || [];

      const incomingPayments = allPayments.filter(
        (p) => p.payment_type === "Incoming"
      );
      const outgoingPayments = allPayments.filter(
        (p) => p.payment_type === "Outgoing"
      );

      const totalIncomingAmount = incomingPayments.reduce(
        (sum, p) => sum + (p.amount_sat || 0),
        0
      );
      const totalOutgoingAmount = outgoingPayments.reduce(
        (sum, p) => sum + (p.amount_sat || 0),
        0
      );

      return {
        incomingCount: incomingPayments.length,
        outgoingCount: outgoingPayments.length,
        totalIncomingAmount,
        totalOutgoingAmount,
        allCount: allPayments.length,
      };
    },
    staleTime: 30000, // 30 seconds
    placeholderData: keepPreviousData, // Show stale data while fetching
  });
}

/**
 * Fetches paginated payments for table display
 */
export function usePayments(
  page: number,
  perPage: number,
  selectedState: string,
  filters?: PaymentFilters
) {
  return useQuery({
    queryKey: ["payments", { page, perPage, selectedState, filters }],
    queryFn: async () => {
      const params = new URLSearchParams();
      params.set("page", String(page));
      params.set("per_page", String(perPage));

      if (selectedState && selectedState !== "all") {
        params.set("payment_types", selectedState);
      }
      if (filters?.paymentState) {
        params.set("states", filters.paymentState);
      }
      if (filters?.operator) params.set("operator", filters.operator);
      if (typeof filters?.value === "number")
        params.set("value", String(filters.value));
      if (filters?.from) params.set("from", filters.from);
      if (filters?.to) params.set("to", filters.to);

      const res = await fetch(`/api/payments?${params.toString()}`);
      if (!res.ok) {
        throw new Error(`Failed to fetch payments: ${res.status}`);
      }

      const data = await res.json();
      return {
        payments: (data?.data?.items || []) as Payment[],
        totalPages: data.pagination?.total_pages || 1,
        totalItems: data.pagination?.total_items || 0,
      };
    },
    staleTime: 30000,
    placeholderData: keepPreviousData,
  });
}
