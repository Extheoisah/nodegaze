"use client";

import { usePathname } from "next/navigation";

export function PageHeader() {
  const pathname = usePathname();
  const last = pathname.split("/").pop() ?? "";
  const pageTitle = last ? last.charAt(0).toUpperCase() + last.slice(1) : "";

  if (!pageTitle) return null;

  return (
    <div className="flex items-center justify-between mb-2 mt-4 font-clash-grotesk text-grey-dark">
      <h1 className="text-3xl font-medium">{pageTitle}</h1>
    </div>
  );
}
