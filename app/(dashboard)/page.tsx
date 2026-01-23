"use client";

import Link from "next/link";
import Image from "next/image";
import { FileText, Plus, Database, TrendingUp, DollarSign, Globe, Settings, ExternalLink, ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from "@/components/ui/card";
import { useInvoices } from "@/hooks/useInvoices";
import { openExternalLink } from "@/lib/tauri/commands";
import { useAuth } from "@/app/contexts/AuthContext";
import { useEffect, useState } from "react";
import { getInvoiceDetail } from "@/lib/tauri/commands";

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat("ro-RO", {
    style: "decimal",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount) + " RON";
}

export default function HomePage() {
  const { invoices } = useInvoices();
  const { isAdmin, isAgent } = useAuth();
  const [totalQuantity, setTotalQuantity] = useState(0);

  // Filter invoices based on role
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const displayInvoices = isAgent
    ? invoices.filter((invoice) => {
      const invoiceDate = new Date(invoice.created_at);
      invoiceDate.setHours(0, 0, 0, 0);
      return invoiceDate.getTime() === today.getTime();
    })
    : invoices; // Admin sees all invoices

  // Calculate total quantity from invoice items
  useEffect(() => {
    const fetchQuantities = async () => {
      let total = 0;
      for (const invoice of displayInvoices) {
        try {
          const detail = await getInvoiceDetail(invoice.id);
          const invoiceQty = detail.items.reduce((sum, item) => sum + item.quantity, 0);
          total += invoiceQty;
        } catch (error) {
          console.error("Error fetching invoice detail:", error);
        }
      }
      setTotalQuantity(total);
    };

    if (displayInvoices.length > 0) {
      fetchQuantities();
    } else {
      setTotalQuantity(0);
    }
  }, [displayInvoices]);

  // Calculate totals
  const totals = displayInvoices.reduce(
    (acc, invoice) => {
      const totalWithoutVAT = invoice.total_amount;
      const estimatedVAT = totalWithoutVAT * 0.19;
      const totalWithVAT = totalWithoutVAT + estimatedVAT;

      return {
        withoutVAT: acc.withoutVAT + totalWithoutVAT,
        withVAT: acc.withVAT + totalWithVAT,
      };
    },
    { withoutVAT: 0, withVAT: 0 }
  );

  const stats = {
    total: displayInvoices.length,
    pending: displayInvoices.filter((i) => i.status === "pending").length,
    sent: displayInvoices.filter((i) => i.status === "sent").length,
    failed: displayInvoices.filter((i) => i.status === "failed").length,
  };

  return (
    <div className="space-y-2 pb-3 max-w-7xl mx-auto w-full">
      {/* Welcome Section - Compact */}
      <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between bg-gradient-to-r from-primary/10 via-transparent to-transparent p-3 rounded-lg">
        <div className="flex items-center gap-2">
          <div className="relative h-30 w-30 flex-shrink-0 hidden sm:block">
            <Image
              src="/logo-simbol-transparent.png"
              alt="eSoft Logo"
              fill
              className="object-contain"
              priority
            />
          </div>
          <div className="flex flex-col">
            <h1 className="text-xl font-bold tracking-tight text-foreground">Bine ai venit!</h1>
            <p className="text-m text-muted-foreground">
              {isAdmin ? "Panou Administrator - Toate datele" : "Panou Agent - Date zilnice"}
            </p>
          </div>
        </div>
        <Link href="/invoices/new">
          <Button size="lg" className="gap-3 h-16 px-7 text-xl">
            <Plus className="h-6 w-6" />
            Factură Nouă
          </Button>
        </Link>
      </div>

      {/* Stats Grid - Compact */}
      <div className="space-y-2">
        <h2 className="text-m font-semibold tracking-tight">
          {isAdmin ? "Sumar Total" : "Sumar Ziua Curentă"}
        </h2>
        {/* Auto-fill grid for better responsiveness on all screen sizes */}
        <div className="grid gap-1.5 grid-cols-2 lg:grid-cols-4">
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-0.5 p-1.5">
              <CardTitle className="text-m font-medium">Facturi {isAgent && "Astăzi"}</CardTitle>
              <div className="h-8 w-9 rounded-full bg-primary/10 flex items-center justify-center">
                <FileText className="h-5 w-5 text-primary" />
              </div>
            </CardHeader>
            <CardContent className="pb-1 px-1.5">
              <div className="text-2xl font-bold">{stats.total}</div>
              <p className="text-base text-muted-foreground">
                {isAdmin ? "Total facturi" : "Emise azi"}
              </p>
            </CardContent>
          </Card>

          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-0.5 p-1.5">
              <CardTitle className="text-m font-medium">În Așteptare</CardTitle>
              <div className="h-8 w-9 rounded-full bg-yellow-500/10 flex items-center justify-center">
                <TrendingUp className="h-5 w-5 text-yellow-600" />
              </div>
            </CardHeader>
            <CardContent className="pb-1 px-1.5">
              <div className="text-2xl font-bold text-yellow-600">{stats.pending}</div>
              <p className="text-base text-muted-foreground">Necesită sincronizare</p>
            </CardContent>
          </Card>

          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-0.5 p-1.5">
              <CardTitle className="text-m font-medium">Trimise</CardTitle>
              <div className="h-8 w-9 rounded-full bg-green-500/10 flex items-center justify-center">
                <TrendingUp className="h-5 w-5 text-green-600" />
              </div>
            </CardHeader>
            <CardContent className="pb-1 px-1.5">
              <div className="text-2xl font-bold text-green-600">{stats.sent}</div>
              <p className="text-base text-muted-foreground">Confirmate WinMentor</p>
            </CardContent>
          </Card>

          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-0.5 p-1.5">
              <CardTitle className="text-m font-medium">Eșuate</CardTitle>
              <div className="h-8 w-9 rounded-full bg-red-500/10 flex items-center justify-center">
                <TrendingUp className="h-5 w-5 text-red-600" />
              </div>
            </CardHeader>
            <CardContent className="pb-1 px-1.5">
              <div className="text-2xl font-bold text-red-600">{stats.failed}</div>
              <p className="text-base text-muted-foreground">Erori validare</p>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Financial Summary - Compact */}
      <Card className="overflow-hidden border-primary/20 bg-muted/5">
        <CardHeader className="bg-muted/30 border-b pb-1 p-1.5">
          <div className="flex items-center gap-2">
            <div className="p-0.5 bg-primary/10 rounded-lg">
              <DollarSign className="h-4 w-4 text-primary" />
            </div>
            <div>
              <CardTitle className="text-base">Totaluri Financiare</CardTitle>
            </div>
          </div>
        </CardHeader>
        <CardContent className="p-1">
          <div className="grid gap-1 grid-cols-1 sm:grid-cols-3">
            <div className="flex flex-col gap-0.5 p-1 bg-background rounded-lg border shadow-sm">
              <span className="text-base text-muted-foreground uppercase tracking-wider font-semibold">Fără TVA</span>
              <span className="text-xl font-bold text-foreground">
                {formatCurrency(totals.withoutVAT)}
              </span>
            </div>
            <div className="flex flex-col gap-0.5 p-1 bg-background rounded-lg border shadow-sm border-l-4 border-l-green-500">
              <span className="text-base text-muted-foreground uppercase tracking-wider font-semibold">Total cu TVA</span>
              <span className="text-xl font-bold text-green-700">
                {formatCurrency(totals.withVAT)}
              </span>
            </div>
            <div className="flex flex-col gap-0.5 p-1 bg-background rounded-lg border shadow-sm">
              <span className="text-base text-muted-foreground uppercase tracking-wider font-semibold">Volum Doc.</span>
              <span className="text-xl font-bold">
                {totalQuantity}
              </span>
              <span className="text-base text-muted-foreground">
                {isAdmin ? "Total articole vândute" : "Articole vândute azi"}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Navigation / Quick Links - Compact */}
      <div className="space-y-2">
        <h2 className="text-m font-semibold tracking-tight">Navigare Rapidă</h2>
        <div className="grid gap-4 grid-cols-2 lg:grid-cols-4 justify-items-center">
          <Link href="/invoices" className="group w-full">
            <Card className="h-full hover:border-primary/50 hover:bg-primary/5 transition-all cursor-pointer group-hover:-translate-y-1 flex flex-col">
              <CardHeader className="pb-1.5 p-3 flex-1">
                <CardTitle className="flex items-center gap-2 text-m">
                  <FileText className="h-5 w-5 text-primary" />
                  Facturi
                </CardTitle>
                <CardDescription className="text-m">Istoric și status</CardDescription>
              </CardHeader>
              <CardFooter className="pt-0 pb-2 px-3">
                <span className="text-m text-primary font-medium flex items-center gap-1 group-hover:underline">
                  Deschide <ArrowRight className="h-2.5 w-2.5" />
                </span>
              </CardFooter>
            </Card>
          </Link>

          <Link href="/data" className="group w-full">
            <Card className="h-full hover:border-primary/50 hover:bg-primary/5 transition-all cursor-pointer group-hover:-translate-y-1 flex flex-col">
              <CardHeader className="pb-1.5 p-3 flex-1">
                <CardTitle className="flex items-center gap-2 text-m">
                  <Database className="h- w-5 text-blue-600" />
                  Date / Parteneri
                </CardTitle>
                <CardDescription className="text-m">Clienți și produse</CardDescription>
              </CardHeader>
              <CardFooter className="pt-0 pb-2 px-3">
                <span className="text-m text-primary font-medium flex items-center gap-1 group-hover:underline">
                  Deschide <ArrowRight className="h-2.5 w-2.5" />
                </span>
              </CardFooter>
            </Card>
          </Link>

          {isAdmin && (
            <Link href="/settings" className="group w-full">
              <Card className="h-full hover:border-primary/50 hover:bg-primary/5 transition-all cursor-pointer group-hover:-translate-y-1 flex flex-col">
                <CardHeader className="pb-1.5 p-3 flex-1">
                  <CardTitle className="flex items-center gap-2 text-m">
                    <Settings className="h-5 w-5 text-slate-600" />
                    Configurare
                  </CardTitle>
                  <CardDescription className="text-m">Setări aplicație</CardDescription>
                </CardHeader>
                <CardFooter className="pt-0 pb-2 px-3">
                  <span className="text-m text-primary font-medium flex items-center gap-1 group-hover:underline">
                    Deschide <ArrowRight className="h-2.5 w-2.5" />
                  </span>
                </CardFooter>
              </Card>
            </Link>
          )}

          <div onClick={() => openExternalLink("https://www.softconsulting.ro/")} className="group cursor-pointer w-full">
            <Card className="h-full border-blue-200 bg-blue-50/50 hover:bg-blue-100/50 hover:border-blue-400 transition-all group-hover:-translate-y-1 flex flex-col">
              <CardHeader className="pb-1.5 p-3 flex-1">
                <CardTitle className="flex items-center gap-2 text-blue-800 text-base">
                  <Globe className="h-5 w-5" />
                  Web eSoft
                </CardTitle>
                <CardDescription className="text-blue-600/80 text-sm">Soluții software</CardDescription>
              </CardHeader>
              <CardFooter className="pt-0 pb-1.5 px-2">
                <span className="text-sm text-blue-700 font-medium flex items-center gap-1 group-hover:underline">
                  Vizitează <ExternalLink className="h-3.5 w-3.5" />
                </span>
              </CardFooter>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
}
