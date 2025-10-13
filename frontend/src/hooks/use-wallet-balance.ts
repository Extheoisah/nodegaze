import { useQuery } from "@tanstack/react-query";
import { keepPreviousData } from "@tanstack/react-query";

export type WalletBalance = {
  confirmed_balance_sat: number;
};

/**
 * Fetches onchain wallet balance
 */
export function useWalletBalance() {
  return useQuery({
    queryKey: ["node", "wallet", "balance"],
    queryFn: async () => {
      const res = await fetch("/api/node/wallet");
      if (!res.ok) {
        throw new Error("Failed to fetch wallet balance");
      }
      const data = await res.json();
      return data?.data?.confirmed_balance_sat || 0;
    },
    staleTime: 30000,
    placeholderData: keepPreviousData,
  });
}
