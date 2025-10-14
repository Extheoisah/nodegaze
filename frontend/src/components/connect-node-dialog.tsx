"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useSession } from "next-auth/react";
import { useQueryClient } from '@tanstack/react-query';

interface ConnectNodeDialogProps {
  onSuccess?: () => void;
}

export function ConnectNodeDialog({ onSuccess }: ConnectNodeDialogProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const { update } = useSession();
  const queryClient = useQueryClient();

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsLoading(true);
    setError("");

    const formData = new FormData(e.currentTarget);
    const data = {
      nodePublicKey: formData.get("nodePublicKey") as string,
      nodeAddress: formData.get("nodeAddress") as string,
      macaroonPath: formData.get("macaroonPath") as string,
      tlsCertPath: formData.get("tlsCertPath") as string,
    };

    try {
      const response = await fetch("/api/node/connect", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
      });

      const result = await response.json();

      if (!response.ok) {
        setError(result.error || "Failed to connect node");
        return;
      }

      if (result.success && result.data?.new_access_token) {
        await update({ accessToken: result.data.new_access_token });
      }

      setIsOpen(false);
      if (onSuccess) {
        onSuccess();
        await queryClient.invalidateQueries();
      }
    } catch (error) {
      setError("An unexpected error occurred");
      console.error("Connect node error:", error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button className="bg-blue-primary hover:bg-blue-600">
          Connect Node
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[600px] font-clash-grotesk">
        <DialogHeader>
          <DialogTitle className="text-2xl font-semibold text-grey-dark">
            Connect Your Lightning Node
          </DialogTitle>
          <DialogDescription>
            Enter your node credentials below to connect and monitor your
            Lightning node.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="flex flex-col gap-4 py-4">
            {error && (
              <div className="p-3 text-sm text-red-600 bg-red-50 rounded-md border border-red-200">
                {error}
              </div>
            )}
            <div className="grid gap-2">
              <Label htmlFor="nodePublicKey">Node Public Key</Label>
              <Input
                id="nodePublicKey"
                name="nodePublicKey"
                type="text"
                placeholder="026c62282d38ea38daa437041b38e696f245749820343f60800c898274e8189467"
                required
                disabled={isLoading}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="nodeAddress">Node Address</Label>
              <Input
                id="nodeAddress"
                name="nodeAddress"
                type="text"
                placeholder="https://192.168.122.92:10001"
                required
                disabled={isLoading}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="macaroonPath">Macaroon Path</Label>
              <Input
                id="macaroonPath"
                name="macaroonPath"
                type="text"
                placeholder="/home/user/.lnd/data/chain/bitcoin/mainnet/admin.macaroon"
                required
                disabled={isLoading}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="tlsCertPath">TLS Certificate Path</Label>
              <Input
                id="tlsCertPath"
                name="tlsCertPath"
                type="text"
                placeholder="/home/user/.lnd/tls.cert"
                required
                disabled={isLoading}
              />
            </div>
          </div>
          <div className="flex justify-end gap-3">
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsOpen(false)}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              className="bg-blue-primary hover:bg-blue-600"
              disabled={isLoading}
            >
              {isLoading ? "Connecting..." : "Connect Node"}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
