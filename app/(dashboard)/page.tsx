"use client";

import Link from "next/link";
import { FileText, Plus, Database, TrendingUp, DollarSign, Package, Globe, Settings, ExternalLink, ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from "@/components/ui/card";
import { useInvoices } from "@/hooks/useInvoices";
import { openExternalLink } from "@/lib/tauri/commands";

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat("ro-RO", {
    style: "decimal",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount) + " RON";
}

export default function HomePage() {
  const { invoices } = useInvoices();

  // Filter invoices for today only
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const todayInvoices = invoices.filter((invoice) => {
    const invoiceDate = new Date(invoice.created_at);
    invoiceDate.setHours(0, 0, 0, 0);
    return invoiceDate.getTime() === today.getTime();
  });

  // Calculate totals for today's invoices
  const todayTotals = todayInvoices.reduce(
    (acc, invoice) => {
      // For now, we calculate without items - will need to fetch items for accurate totals
      const totalWithoutVAT = invoice.total_amount;
      const estimatedVAT = totalWithoutVAT * 0.19; // Assuming 19% VAT
      const totalWithVAT = totalWithoutVAT + estimatedVAT;

      return {
        withoutVAT: acc.withoutVAT + totalWithoutVAT,
        withVAT: acc.withVAT + totalWithVAT,
        quantities: acc.quantities + 1, // Count number of invoices as proxy for quantities
      };
    },
    { withoutVAT: 0, withVAT: 0, quantities: 0 }
  );

  const stats = {
    total: todayInvoices.length,
    pending: todayInvoices.filter((i) => i.status === "pending").length,
    sent: todayInvoices.filter((i) => i.status === "sent").length,
    failed: todayInvoices.filter((i) => i.status === "failed").length,
  };

  return (
    <div className="space-y-8 pb-10">
      {/* Welcome Section */}
      <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between bg-gradient-to-r from-primary/10 via-transparent to-transparent p-6 rounded-lg">
        <div className="flex flex-col">
          <h1 className="text-3xl font-bold tracking-tight text-foreground">Bine ai venit!</h1>
          <p className="text-muted-foreground mt-1 max-w-md">
            Panoul tău de control pentru gestiunea facturilor și a partenerilor
          </p>
        </div>
        <Link href="/invoices/new">
          <Button size="lg" className="gap-2 shadow-lg hover:shadow-primary/20 transition-all">
            <Plus className="h-5 w-5" />
            Factură Nouă
          </Button>
        </Link>
      </div>

      {/* Stats Grid */}
      <div className="space-y-4">
        <h2 className="text-lg font-semibold tracking-tight">Sumar Ziua Curentă</h2>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Facturi Astăzi</CardTitle>
              <div className="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center">
                <FileText className="h-4 w-4 text-primary" />
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.total}</div>
              <p className="text-xs text-muted-foreground mt-1">Toate facturile emise azi</p>
            </CardContent>
          </Card>

          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">În Așteptare</CardTitle>
              <div className="h-8 w-8 rounded-full bg-yellow-500/10 flex items-center justify-center">
                <TrendingUp className="h-4 w-4 text-yellow-600" />
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-yellow-600">{stats.pending}</div>
              <p className="text-xs text-muted-foreground mt-1">Necesită sincronizare</p>
            </CardContent>
          </Card>

          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Trimise cu Succes</CardTitle>
              <div className="h-8 w-8 rounded-full bg-green-500/10 flex items-center justify-center">
                <TrendingUp className="h-4 w-4 text-green-600" />
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-green-600">{stats.sent}</div>
              <p className="text-xs text-muted-foreground mt-1">Confirmate de WinMentor</p>
            </CardContent>
          </Card>

          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Eșuate / Offline</CardTitle>
              <div className="h-8 w-8 rounded-full bg-red-500/10 flex items-center justify-center">
                <TrendingUp className="h-4 w-4 text-red-600" />
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-red-600">{stats.failed}</div>
              <p className="text-xs text-muted-foreground mt-1">Erori critice de validare</p>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Financial Summary */}
      <Card className="overflow-hidden border-primary/20 bg-muted/5">
        <CardHeader className="bg-muted/30 border-b">
          <div className="flex items-center gap-2">
            <div className="p-2 bg-primary/10 rounded-lg">
              <DollarSign className="h-5 w-5 text-primary" />
            </div>
            <div>
              <CardTitle>Totaluri Financiare</CardTitle>
              <CardDescription>Valoarea facturilor emise astăzi</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="p-6">
          <div className="grid gap-6 md:grid-cols-3">
            <div className="flex flex-col gap-1 p-4 bg-background rounded-lg border shadow-sm">
              <span className="text-sm text-muted-foreground uppercase tracking-wider font-semibold">Fără TVA</span>
              <span className="text-2xl font-bold text-foreground">
                {formatCurrency(todayTotals.withoutVAT)}
              </span>
            </div>
            <div className="flex flex-col gap-1 p-4 bg-background rounded-lg border shadow-sm border-l-4 border-l-green-500">
              <span className="text-sm text-muted-foreground uppercase tracking-wider font-semibold">Total cu TVA</span>
              <span className="text-2xl font-bold text-green-700">
                {formatCurrency(todayTotals.withVAT)}
              </span>
            </div>
            <div className="flex flex-col gap-1 p-4 bg-background rounded-lg border shadow-sm">
              <span className="text-sm text-muted-foreground uppercase tracking-wider font-semibold">Volum Doc.</span>
              <span className="text-2xl font-bold">
                {todayTotals.quantities}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Navigation / Quick Links */}
      <div className="space-y-4">
        <h2 className="text-lg font-semibold tracking-tight">Navigare Rapidă</h2>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <Link href="/invoices" className="group">
            <Card className="h-full hover:border-primary/50 hover:bg-primary/5 transition-all cursor-pointer group-hover:-translate-y-1">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <FileText className="h-5 w-5 text-primary" />
                  Facturi
                </CardTitle>
                <CardDescription>Istoric și status facturi</CardDescription>
              </CardHeader>
              <CardFooter className="pt-0">
                <span className="text-xs text-primary font-medium flex items-center gap-1 group-hover:underline">Deschide <ArrowRight className="h-3 w-3" /></span>
              </CardFooter>
            </Card>
          </Link>

          <Link href="/data" className="group">
            <Card className="h-full hover:border-primary/50 hover:bg-primary/5 transition-all cursor-pointer group-hover:-translate-y-1">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Database className="h-5 w-5 text-blue-600" />
                  Date / Parteneri
                </CardTitle>
                <CardDescription>Clienți, produse și prețuri</CardDescription>
              </CardHeader>
              <CardFooter className="pt-0">
                <span className="text-xs text-primary font-medium flex items-center gap-1 group-hover:underline">Deschide <ArrowRight className="h-3 w-3" /></span>
              </CardFooter>
            </Card>
          </Link>

          <Link href="/settings" className="group">
            <Card className="h-full hover:border-primary/50 hover:bg-primary/5 transition-all cursor-pointer group-hover:-translate-y-1">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Settings className="h-5 w-5 text-slate-600" />
                  Configurare
                </CardTitle>
                <CardDescription>Setări aplicație și utilizator</CardDescription>
              </CardHeader>
              <CardFooter className="pt-0">
                <span className="text-xs text-primary font-medium flex items-center gap-1 group-hover:underline">Deschide <ArrowRight className="h-3 w-3" /></span>
              </CardFooter>
            </Card>
          </Link>

          <div onClick={() => openExternalLink("https://www.softconsulting.ro/")} className="group cursor-pointer">
            <Card className="h-full border-blue-200 bg-blue-50/50 hover:bg-blue-100/50 hover:border-blue-400 transition-all group-hover:-translate-y-1">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-blue-800">
                  <Globe className="h-5 w-5" />
                  Web eSoft
                </CardTitle>
                <CardDescription className="text-blue-600/80">Soluții software integrate</CardDescription>
              </CardHeader>
              <CardFooter className="pt-0">
                <span className="text-xs text-blue-700 font-medium flex items-center gap-1 group-hover:underline">Vizitează <ExternalLink className="h-3 w-3" /></span>
              </CardFooter>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
}
