import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// ==================== FORMATTING UTILITIES ====================

/**
 * Rounds a monetary amount to 2 decimals (bani).
 *
 * Any amount sent to the backend must go through this. WME keeps money at 2 decimals, so an
 * unrounded value such as 78.82 * 1.09 = 85.9138 reaches the ERP as "85,914" and cannot be
 * reconciled against the invoice it pays.
 */
export function round2(amount: number): number {
  return Math.round(amount * 100) / 100;
}

export function formatCurrency(amount: number): string {
  return new Intl.NumberFormat("ro-RO", {
    style: "decimal",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount) + " RON";
}

export function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString("ro-RO", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
  });
}

export function formatDateTime(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString("ro-RO", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleTimeString("ro-RO", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function calculateDueDate(createdAt: string, paymentTermDays: number = 7): string {
  const date = new Date(createdAt);
  date.setDate(date.getDate() + paymentTermDays);
  return formatDate(date.toISOString());
}
