"use client";

import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { InvoiceWizard } from "@/app/components/invoices/wizard/InvoiceWizard";

export default function NewInvoicePage() {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <Link href="/invoices">
          <Button variant="ghost" size="icon" className="h-10 w-10">
            <ArrowLeft className="h-5 w-5" />
          </Button>
        </Link>
        <div>
          <h1 className="text-2xl font-bold">Factură nouă</h1>
          <p className="text-muted-foreground">
            Creează o nouă factură pentru un partener
          </p>
        </div>
      </div>

      {/* Wizard */}
      <InvoiceWizard />
    </div>
  );
}
