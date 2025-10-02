"use client";
// import { use } from "react";
import { useEffect } from "react";
import React from "react";

import { AppLayout } from "@/components/app-layout";
import { Button } from "@/components/ui/button";
import { ArrowLeftIcon } from "@/public/assets/icons/arrow-left";
import { Copy, DatabaseIcon } from "lucide-react";
import Link from "next/link";

type ChannelDetailsPageProps = {
  params: Promise<{ id: string }>;
};

interface ConnectedNode {
  peer: string;
  publicKey: string;
  feeRate: string;
  baseFee: string;
  maxHTLC: string;
  minHTLC: string;
  timelockDelta: string;
  disabled: string;
}

interface ChannelData {
  channelId: string;
  inboundBalance: number;
  outboundBalance: number;
  channelAge: string;
  lastUpdated: string;
  capacity: number;
  openingCost: number;
  commitmentTransactionId: string;
  connectedNodes: ConnectedNode[];
}

// export default function ChannelDetailsPage(props: ChannelDetailsPageProps) {
//   const params = use(props.params);
//   const { id, inbound_balance, channel_name } = params;

export default function ChannelDetailsPage({
  params,
}: ChannelDetailsPageProps) {
  const { id } = React.use(params); 

  const [channelData, setChannelData] = React.useState<ChannelData | null>(
    null
  );

  useEffect(() => {
    async function fetchChannelData() {
      try {
        const res = await fetch(`/api/channels/${id}`);
        const json = await res.json();
         console.log(json); // debug

        const channel = json.data; 
        
        setChannelData({
          channelId: channel.channel_id,
          inboundBalance: channel.remote_balance_sat,
          outboundBalance: channel.local_balance_sat,
          channelAge: `${Math.floor(channel.channel_age_blocks / 3600)} days`, 
          lastUpdated: new Date(json.timestamp).toLocaleString(),

          capacity: channel.capacity_sat,
          openingCost: channel.opening_cost_sat ?? 0,
          commitmentTransactionId: channel.txid,
          connectedNodes: [
            {
              peer: "Node 1",
              publicKey: channel.node1_policy.pubkey,
              feeRate: `${channel.node1_policy.fee_rate_milli_msat} ppm`,
              baseFee: `${channel.node1_policy.fee_base_msat / 1000} sats`,
              maxHTLC: `${channel.node1_policy.max_htlc_msat / 1000} sats`,
              minHTLC: `${channel.node1_policy.min_htlc_msat / 1000} sats`,
              timelockDelta: `${
                channel.node1_policy.time_lock_delta ?? 0
              } blocks`,
              disabled: channel.node1_policy.disabled ? "Yes" : "No",
            },
            {
              peer: "Node 2",
              publicKey: channel.node2_policy.pubkey,
              feeRate: `${channel.node2_policy.fee_rate_milli_msat} ppm`,
              baseFee: `${channel.node2_policy.fee_base_msat / 1000} sats`,
              maxHTLC: `${channel.node2_policy.max_htlc_msat / 1000} sats`,
              minHTLC: `${channel.node2_policy.min_htlc_msat / 1000} sats`,
              timelockDelta: `${
                channel.node2_policy.time_lock_delta ?? 0
              } blocks`,
              disabled: channel.node2_policy.disabled ? "Yes" : "No",
            },
          ],
        });
      } catch (error) {
        console.error("Failed to load channel data", error);
      }
    }

    fetchChannelData();
  }, [id]);
  
  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  // if (!channelData) {
  //   return <div>Loading...</div>;
  // }

  return (
    <AppLayout>
      {/* Breadcrumb */}
      <div className="flex items-center gap-2 text-sm mb-6">
        <span className="text-grey-accent">
          <Link href="/channels">Channels</Link>
        </span>
        <span className="text-grey-accent">&gt;</span>
        <span className="text-grey-dark font-medium">Channel Details</span>
        <span className="text-grey-accent">&gt;</span>
        <span className="text-blue-primary font-medium">{id}</span>
      </div>

      {/* Back Button */}
      <div className="h-fit">
        <Link
          href="/channels"
          className="flex items-center gap-2 font-medium w-fit mb-4 pl-0 h-auto text-grey-dark text-sm hover:text-grey-dark"
        >
          <ArrowLeftIcon className="h-4 w-4 text-grey-dark" />
          Back
        </Link>
      </div>

      {/* Page Title */}
      <h1 className="text-3xl font-medium text-grey-dark mb-8">
        Channel Details
      </h1>

      {/* Balance Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8 max-w-6xl">
        {/* Inbound Balance Card */}
        <div className="bg-white rounded-xl border p-6">
          <div className="flex items-center gap-3 mb-2">
            <div className="w-7 h-7 bg-cerulean-blue-accent rounded-full flex items-center justify-center">
              {/* <div className="w-3 h-3 bg-blue-500 rounded"></div> */}
              <DatabaseIcon className="h-3 w-3 text-blue-primary" />
            </div>
            <span className="text-grey-accent font-medium">
              Inbound Balance
            </span>
          </div>
          <div className="flex items-center gap-2 mb-4">
            <span className="text-success-green text-sm">↑ 7.2%</span>
          </div>
          <div className="text-3xl font-medium text-grey-dark mb-4">
            {channelData
              ? new Intl.NumberFormat("en-US").format(channelData.inboundBalance)
              : ""}{" "}
            sats
          </div>
          {/* Mini chart placeholder */}
          <div className="h-16 bg-success-green-background rounded-lg flex items-end justify-center">
            <div className="flex items-end gap-1 h-full py-2">
              {[4, 6, 3, 8, 5, 9, 7, 6, 8, 10, 7, 9].map((height, i) => (
                <div
                  key={i}
                  className="w-2 bg-success-green rounded-sm"
                  style={{ height: `${height * 4}px` }}
                />
              ))}
            </div>
          </div>
        </div>

        {/* Outbound Balance Card */}
        <div className="bg-white rounded-xl border p-6">
          <div className="flex items-center gap-3 mb-2">
            <div className="w-7 h-7 bg-red-100 rounded-full flex items-center justify-center">
              <DatabaseIcon className="h-3 w-3 text-red-500" />
            </div>
            <span className="text-grey-accent font-medium">
              Outbound Balance
            </span>
          </div>
          <div className="flex items-center gap-2 mb-4">
            <span className="text-red-500 text-sm">↓ 7.2%</span>
          </div>
          <div className="text-3xl font-medium text-grey-dark mb-4">
            {channelData
              ? new Intl.NumberFormat("en-US").format(channelData.outboundBalance)
              : ""}{" "}
            sats
          </div>
          {/* Mini chart placeholder */}
          <div className="h-16 bg-red-50 rounded-lg flex items-end justify-center">
            <div className="flex items-end gap-1 h-full py-2">
              {[8, 6, 9, 4, 7, 5, 3, 6, 4, 2, 5, 3].map((height, i) => (
                <div
                  key={i}
                  className="w-2 bg-red-400 rounded-sm"
                  style={{ height: `${height * 4}px` }}
                />
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Channel Details Section */}
      <div className="bg-white rounded-xl border p-6 mb-8">
        <h2 className="text-base font-medium text-grey-dark mb-6">
          Channel Details
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-4">
            <div>
              <div className="text-sm text-grey-accent mb-1">Channel ID</div>
              <div className="text-base font-medium text-maya-blue">
                {channelData ? channelData.channelId : "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Channel Age</div>
              <div className="text-base font-medium text-maya-blue">
                {channelData ? channelData.channelAge : "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Capacity</div>
              <div className="text-base font-medium text-maya-blue">
                {channelData
                  ? new Intl.NumberFormat("en-US").format(channelData.capacity) : "..."} sats
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <div className="text-sm text-grey-accent mb-1">
                Commitment Transaction
              </div>
              <div className="flex items-center gap-2">
                <div className="font-medium text-maya-blue font-mono text-base truncate max-w-64">
                  {/* {channelData.commitmentTransactionId} */}
                  {channelData ? channelData.commitmentTransactionId : "..."}
                </div>
                <Button
                  variant="link"
                  size="sm"
                  onClick={() =>
                    copyToClipboard(channelData ? channelData.commitmentTransactionId : "...")
                  }
                  className="p-1 h-6 w-6"
                >
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Last Updated</div>
              <div className="text-base font-medium text-maya-blue">
                {channelData ? channelData.lastUpdated : "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Opening Cost</div>
              <div className="text-base font-medium text-maya-blue">
                {channelData ? new Intl.NumberFormat("en-US").format(channelData.openingCost) : "..."} sats
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Connected Node Details */}
      <div className="bg-white rounded-xl border p-6">
        <h2 className="text-base font-medium text-grey-dark mb-6">
          Connected Node Details
        </h2>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {channelData?.connectedNodes?.map((node, index) => (
            <div key={index} className="space-y-4">
              {/* <div> */}
              <div className="text-sm text-grey-accent mb-1">
                Peer {index + 1}
              </div>
              <div className="text-base font-medium text-maya-blue">
                {node.peer}
              </div>
              {/* </div> */}

              {/* <div> */}
              <div className="text-sm text-grey-accent mb-1">Public Key</div>
              <div className="flex items-center gap-2">
                <div className="text-base font-mono text-maya-blue truncate max-w-64">
                  {node.publicKey}
                </div>
                <Button
                  variant="link"
                  size="sm"
                  onClick={() => copyToClipboard(node.publicKey)}
                  className="p-1 h-6 w-6"
                >
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
              {/* </div> */}

              {/* <div className="grid grid-cols-2 gap-4"> */}
              <div>
                <div className="text-sm text-grey-accent mb-1">Fee Rate</div>
                <div className="text-base font-medium text-maya-blue">
                  {node.feeRate}
                </div>
              </div>
              <div>
                <div className="text-sm text-grey-accent mb-1">Base Fee</div>
                <div className="text-base font-medium text-maya-blue">
                  {node.baseFee}
                </div>
              </div>
              {/* </div> */}

              {/* <div className="grid grid-cols-2 gap-4"> */}
              <div>
                <div className="text-sm text-grey-accent mb-1">Max HTLC</div>
                <div className="text-base font-medium text-maya-blue">
                  {node.maxHTLC}
                </div>
              </div>
              <div>
                <div className="text-sm text-grey-accent mb-1">Min HTLC</div>
                <div className="text-base font-medium text-maya-blue">
                  {node.minHTLC}
                </div>
              </div>
              {/* </div> */}

              {/* <div className="grid grid-cols-2 gap-4"> */}
              <div>
                <div className="text-sm text-grey-accent mb-1">
                  Timelock Delta
                </div>
                <div className="text-base font-medium text-maya-blue">
                  {node.timelockDelta}
                </div>
              </div>
              <div>
                <div className="text-sm text-grey-accent mb-1">Disabled</div>
                <div className="text-base font-medium text-maya-blue">
                  {node.disabled}
                </div>
              </div>
              {/* </div> */}
            </div>
          ))}
        </div>
      </div>
    </AppLayout>
  );
}
