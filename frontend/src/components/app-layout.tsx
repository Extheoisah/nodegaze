"use client";

import type * as React from "react";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { DashboardHeader } from "@/components/dashboard-header";
import { usePathname } from "next/navigation";

interface AppLayoutProps {
  children: React.ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  const pathname = usePathname();
  const firstLetter = pathname.split("/").pop()?.charAt(0)?.toUpperCase();
  const rest = pathname.split("/").pop()?.slice(1);
  const pageTitle = firstLetter ? firstLetter + rest : "";
  if (!pageTitle) return null;

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <DashboardHeader pageTitle={pageTitle} />
        <div className="flex flex-1 flex-col gap-6 p-6 font-clash-grotesk text-sidebar">
          {children}
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
} 