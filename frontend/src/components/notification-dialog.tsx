"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";

interface NotificationDialogProps {
  selectedEvent: string;
  onSubmit: (data: {
    eventType: string;
    url: string;
    secret?: string;
    description: string;
  }) => void;
}

export function NotificationDialog({
  selectedEvent,
  onSubmit,
}: NotificationDialogProps) {
  const [webhookUrl, setWebhookUrl] = useState("");
  const [webhookSecret, setWebhookSecret] = useState("");
  const [description, setDescription] = useState("");
  const [isOpen, setIsOpen] = useState(false);

  const handleSubmit = () => {
    // validate webhook url type
    // webhook url should be a valid url
    if (!webhookUrl.startsWith("https://") && selectedEvent === "webhook") {
      alert("Invalid webhook URL");
      return;
    }

    // discord webhook url should be a valid url
    if (
      selectedEvent === "discord" &&
      !webhookUrl.startsWith("https://discord.com/api/webhooks/")
    ) {
      alert("Invalid Discord webhook URL");
      return;
    }

    onSubmit({
      eventType: selectedEvent,
      url: webhookUrl,
      secret: webhookSecret,
      description: description,
    });

    // Reset form
    setWebhookUrl("");
    setWebhookSecret("");
    setDescription("");
    setIsOpen(false);
  };

  const resetForm = () => {
    setWebhookUrl("");
    setWebhookSecret("");
    setDescription("");
  };

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        setIsOpen(open);
        if (!open) resetForm();
      }}
    >
      <DialogTrigger asChild>
        <Button
          disabled={!selectedEvent}
          className={`bg-black-accent hover:bg-blue-600 cursor-pointer font-clash-grotesk rounded-3xl text-white px-8 py-6 disabled:opacity-50 ${
            selectedEvent
              ? "cursor-pointer bg-blue-primary hover:bg-blue-600"
              : "cursor-not-allowed bg-grey-accent"
          }`}
        >
          Connect
        </Button>
      </DialogTrigger>
      <DialogContent className="font-clash-grotesk">
        <DialogHeader>
          <DialogTitle className="text-xl font-medium text-grey-dark">
            Notification Settings
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div>
            <Label className="text-grey-accent text-sm mb-2 font-normal">
              {selectedEvent === "discord"
                ? "Discord Webhook URL"
                : "Webhook URL"}
            </Label>
            <Input
              value={webhookUrl}
              onChange={(e) => setWebhookUrl(e.target.value)}
              placeholder="https://api.your-application.com/webhooks/receive_data_12345"
              className="mt-1 h-12"
            />
          </div>

          {/* Webhook Secret Field - only show for webhook */}
          {selectedEvent === "webhook" && (
            <div>
              <Label className="text-grey-accent text-sm mb-2 font-normal">
                Webhook Secret
              </Label>
              <Input
                value={webhookSecret}
                onChange={(e) => setWebhookSecret(e.target.value)}
                placeholder="Enter webhook secret"
                type="password"
                className="mt-1"
              />
            </div>
          )}

          <div>
            <Label className="text-grey-accent text-sm mb-2 font-normal">
              Description
            </Label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="This is a description for this webhook URL"
              className="mt-1 w-full min-h-[100px] px-3 py-2 text-sm border border-input rounded-md bg-background ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 resize-none"
            />
          </div>

          <Button
            onClick={handleSubmit}
            className={`w-full text-white font-clash-grotesk h-12 rounded-3xl ${
              (selectedEvent === "webhook" && webhookUrl && webhookSecret) ||
              (selectedEvent === "discord" && webhookUrl)
                ? "cursor-pointer bg-blue-primary hover:bg-blue-600"
                : "cursor-not-allowed bg-grey-accent"
            }`}
            disabled={
              !(
                (selectedEvent === "webhook" && webhookUrl && webhookSecret) ||
                (selectedEvent === "discord" && webhookUrl)
              )
            }
          >
            Connect
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
