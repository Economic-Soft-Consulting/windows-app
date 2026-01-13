"use client";

import { Badge } from "@/components/ui/badge";
import { Loader2, Check, X, Clock } from "lucide-react";
import type { InvoiceStatus } from "@/lib/tauri/types";

interface InvoiceStatusBadgeProps {
  status: InvoiceStatus;
}

const statusConfig: Record<InvoiceStatus, {
  label: string;
  variant: "default" | "secondary" | "destructive" | "outline";
  icon: React.ComponentType<{ className?: string }>;
  className: string;
}> = {
  pending: {
    label: "În așteptare",
    variant: "secondary",
    icon: Clock,
    className: "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400 border-yellow-200 dark:border-yellow-800",
  },
  sending: {
    label: "Se trimite...",
    variant: "secondary",
    icon: Loader2,
    className: "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400 border-blue-200 dark:border-blue-800",
  },
  sent: {
    label: "Trimisă",
    variant: "default",
    icon: Check,
    className: "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 border-green-200 dark:border-green-800",
  },
  failed: {
    label: "Eșuată",
    variant: "destructive",
    icon: X,
    className: "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400 border-red-200 dark:border-red-800",
  },
};

export function InvoiceStatusBadge({ status }: InvoiceStatusBadgeProps) {
  const config = statusConfig[status];
  const Icon = config.icon;

  return (
    <Badge variant="outline" className={`gap-1.5 ${config.className}`}>
      <Icon className={`h-3.5 w-3.5 ${status === "sending" ? "animate-spin" : ""}`} />
      {config.label}
    </Badge>
  );
}
