import { useMemo } from "react";
import { useChannels } from "./use-channels";
import { usePayments } from "./use-payments";
import { useWalletBalance } from "./use-wallet-balance";

export type NodeOverviewMetrics = {
  channelCount: number;
  activeChannelCount: number;
  totalCapacitySat: number;
  totalLocalBalanceSat: number; // Outbound
  totalRemoteBalanceSat: number; // Inbound
  nodeAlias?: string;
  totalPayments: number;
  settledPayments: number;
  totalIncomingVolumeSat: number;
  totalOutgoingVolumeSat: number;
  onchainBalanceSat: number;
};

/**
 * Fetches all channels and payments, calculates overview metrics client-side
 * Uses React Query's caching to avoid repeated calculations
 */
export function useNodeOverview() {
  const {
    data: channelsData,
    isLoading: channelsLoading,
    error: channelsError,
  } = useChannels(1, 100);

  const {
    data: paymentsData,
    isLoading: paymentsLoading,
    error: paymentsError,
  } = usePayments(1, 100, "all", undefined);

  const {
    data: walletBalance,
    isLoading: walletLoading,
    error: walletError,
  } = useWalletBalance();

  const metrics = useMemo((): NodeOverviewMetrics | null => {
    if (!channelsData?.channels) return null;

    const channels = channelsData.channels;
    const payments = paymentsData?.payments || [];
    const onchainBalance = walletBalance || 0;

    const activeChannels = channels.filter(
      (c) => c.state.toLowerCase() === "active"
    );

    const totalCapacity = channels.reduce((sum, c) => {
      const capacity = c.inbound_balance + c.outbound_balance;
      return sum + capacity;
    }, 0);

    const totalLocalBalance = channels.reduce(
      (sum, c) => sum + c.outbound_balance,
      0
    );

    const totalRemoteBalance = channels.reduce(
      (sum, c) => sum + c.inbound_balance,
      0
    );

    const settledPayments = payments.filter(
      (p) => p.state.toLowerCase() === "settled"
    );

    const incomingPayments = payments.filter(
      (p) =>
        p.payment_type === "Incoming" && p.state.toLowerCase() === "settled"
    );

    const outgoingPayments = payments.filter(
      (p) =>
        p.payment_type === "Outgoing" && p.state.toLowerCase() === "settled"
    );

    const totalIncomingVolume = incomingPayments.reduce(
      (sum, p) => sum + p.amount_sat,
      0
    );

    const totalOutgoingVolume = outgoingPayments.reduce(
      (sum, p) => sum + p.amount_sat,
      0
    );

    return {
      channelCount: channels.length,
      activeChannelCount: activeChannels.length,
      totalCapacitySat: totalCapacity,
      totalLocalBalanceSat: totalLocalBalance,
      totalRemoteBalanceSat: totalRemoteBalance,
      totalPayments: payments.length,
      settledPayments: settledPayments.length,
      totalIncomingVolumeSat: totalIncomingVolume,
      totalOutgoingVolumeSat: totalOutgoingVolume,
      onchainBalanceSat: onchainBalance,
    };
  }, [channelsData, paymentsData, walletBalance]);

  return {
    metrics,
    isLoading: channelsLoading || paymentsLoading || walletLoading,
    error: channelsError || paymentsError || walletError,
  };
}
