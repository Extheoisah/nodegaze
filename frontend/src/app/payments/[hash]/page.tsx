"use client";
import React, { useEffect } from "react";
import { AppLayout } from "@/components/app-layout";
import { Button } from "@/components/ui/button";
import { ArrowLeftIcon } from "@/public/assets/icons/arrow-left";
import { Copy } from "lucide-react";
import Link from "next/link";

type PaymentDetailsPageProps = {
  params: Promise<{ hash: string }>;
};


interface PaymentData {
  state?: string;
  payment_type?: string;
  amount_sat?: number | string;
  amount_usd?: number | string;
  routing_fee?: number | string;
  network?: string;
  alias?: string;
  public_capacity?: string;
  description?: string | null;
  creation_time?: number | {
    secs_since_epoch: number;
    nanos_since_epoch: number;
  };
  invoice?: string;
  payment_hash?: string;
  completed_at?: number | string;
  paymentId?: string;
  public?: string;
  date?: string;
  expiry?: string | number;
  public_channel?: string;
  last_updated?: number | string;
}

export default function PaymentDetailsPage({ params }: PaymentDetailsPageProps) {
  const { hash } = React.use(params);

  // console.log({hash})
  
  const [paymentData, setPaymentData] = React.useState<PaymentData | null>(null);

  useEffect(() => {
  async function fetchPaymentData() {
    try {
      const res = await fetch(`/api/payments/${hash}`);
      console.log("Fetch response:", res);
      const json = await res.json();
      console.log("API response:", json);

      const payment = json.data;
      
      setPaymentData({
        state: payment.state ?? "...",
        payment_type: payment.payment_type ?? "...",
        amount_sat: payment.amount_sat ?? "...",
        amount_usd: payment.amount_usd ?? "...",
        routing_fee: payment.routing_fee ?? "...",
        network: payment.network ?? "...",
        description: payment.description === "" ? "Null" : (payment.description ?? "Null"),
        invoice: payment.invoice ?? "...",
        payment_hash: payment.payment_hash ?? "...",
        completed_at: payment.completed_at ?? "...",
        public: payment.destination_pubkey ?? "...",
        alias: payment.alias ?? "...",
        public_capacity: payment.public_capacity ?? "...",
        expiry: payment.htlcs?.[0]?.routes?.[0]?.hops?.[0]?.expiry ?? "...",
        public_channel: payment.htlcs?.[0]?.routes?.[0]?.hops?.[0]?.chan_id ?? "...",
        date: (typeof payment.creation_time === "number"
          ? payment.creation_time
          : payment.creation_time?.secs_since_epoch)
          ? new Date(((typeof payment.creation_time === "number"
              ? payment.creation_time
              : payment.creation_time?.secs_since_epoch) as number) * 1000).toLocaleDateString()
          : "...",
        last_updated: payment.last_updated ?? "...",
      });
    } catch (error) {
      console.error("Failed to load payment data", error);
    }
  }

  fetchPaymentData();
}, [hash]);


  const copyToClipboard = (text?: string) => {
    if (!text || text === "...") return;
    navigator.clipboard.writeText(text);
  };

  return (
    <AppLayout>
      {/* Breadcrumb */}
      <div className="flex items-center gap-2 text-sm mb-6">
        <span className="text-grey-accent">
          <Link href="/payments">Payments</Link>
        </span>
        <span className="text-grey-accent">&gt;</span>
        <span className="text-[#204ECF] font-medium">Payment Details</span>

      </div>
      
      {/* Back Button */}
      <div className="h-fit">
        <Link
          href="/payments"
          className="flex items-center gap-2 font-medium w-fit mb-4 pl-0 h-auto text-grey-dark text-sm hover:text-grey-dark"
        >
          <ArrowLeftIcon className="h-4 w-4 text-grey-dark" />
          Back
        </Link>
      </div>

      {/* Page Title */}
      <h1 className="text-3xl font-medium text-grey-dark mb-6">
        Payment Details
      </h1>

      {/* Payment Details Section */}
      <div className="bg-white rounded-xl border p-6 mb-8">
        <h2 className="text-base font-medium text-grey-dark mb-6">
          Invoice Details
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-4">
            <div>
              <div className="text-sm text-grey-accent mb-1">State</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.state ?? "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Amount (sats)</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.amount_sat !== undefined && paymentData?.amount_sat !== "..."
                  ? new Intl.NumberFormat("en-US").format(Number(paymentData.amount_sat))
                  : "..."} sats
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">
                Routing fee (sats)
              </div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.routing_fee !== undefined && paymentData?.routing_fee !== "..."
                  ? new Intl.NumberFormat("en-US").format(Number(paymentData.routing_fee))
                  : "..."} sats
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Network</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.network ?? "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Description</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.description ?? "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Date</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.date ?? "..."}
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <div className="text-sm text-grey-accent mb-1">Type</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.payment_type ?? "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Amount (USD)</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.amount_usd !== undefined && paymentData?.amount_usd !== "..."
                  ? new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" }).format(Number(paymentData.amount_usd))
                  : "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Invoice</div>
              <div className="flex items-center gap-2">
                <div className="font-medium text-maya-blue font-mono text-base truncate max-w-64">
                  {paymentData?.invoice ?? "..."}
                </div>
                <Button
                  variant="link"
                  size="sm"
                  onClick={() => copyToClipboard(paymentData?.invoice)}
                  className="p-1 h-6 w-6"
                >
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Expiry</div>
              <div className="text-base font-medium text-maya-blue">
                expired since {paymentData?.expiry ?? "..."} minutes
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Payment hash</div>
              <div className="flex items-center gap-2">
                <div className="font-medium text-maya-blue font-mono text-base truncate max-w-64">
                  {paymentData?.payment_hash ?? "..."}
                </div>
                <Button
                  variant="link"
                  size="sm"
                  onClick={() => copyToClipboard(paymentData?.payment_hash)}
                  className="p-1 h-6 w-6"
                >
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Destination Node Data */}
      <div className="bg-white rounded-xl border p-6">
        <h2 className="text-base font-medium text-grey-dark mb-6">
          Destination Node Data
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-4">
            <div>
              <div className="text-sm text-grey-accent mb-1">Alias</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.alias ?? "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Public Capacity</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.public_capacity ?? "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Public Key</div>
              <div className="flex items-center gap-2">
                <div className="font-medium text-maya-blue font-mono text-base truncate max-w-64">
                  {paymentData?.public ?? "..."}
                </div>
                <Button
                  variant="link"
                  size="sm"
                  onClick={() => copyToClipboard(paymentData?.public)}
                  className="p-1 h-6 w-6"
                >
                  <Copy className="h-3 w-3" />
                </Button>
              </div>
            </div>
          </div>
          
          <div className="space-y-4">
            <div>
              <div className="text-sm text-grey-accent mb-1">Public Channels</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.public_channel ?? "..."}
              </div>
            </div>
            <div>
              <div className="text-sm text-grey-accent mb-1">Last Update</div>
              <div className="text-base font-medium text-maya-blue">
                {paymentData?.last_updated ?? "..."} minutes ago
              </div>
            </div>
          </div>
        </div>
      </div>
    </AppLayout>
  );
}

