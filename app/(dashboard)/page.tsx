"use client";

import Link from "next/link";
import { FileText, Plus, Database, TrendingUp } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useInvoices } from "@/hooks/useInvoices";

export default function HomePage() {
  const { invoices } = useInvoices();

  const stats = {
    total: invoices.length,
    pending: invoices.filter((i) => i.status === "pending").length,
    sent: invoices.filter((i) => i.status === "sent").length,
    failed: invoices.filter((i) => i.status === "failed").length,
  };

  return (
    <div className="space-y-6">
      {/* Welcome Section */}
      <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Bine ai venit!</h1>
          <p className="text-muted-foreground mt-1">
            Gestionează facturile către partenerii tăi
          </p>
        </div>
        <Link href="/invoices/new">
          <Button className="gap-2">
            <Plus className="h-4 w-4" />
            Factură nouă
          </Button>
        </Link>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Total facturi
            </CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              În așteptare
            </CardTitle>
            <TrendingUp className="h-4 w-4 text-yellow-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-yellow-600">{stats.pending}</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Trimise
            </CardTitle>
            <TrendingUp className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">{stats.sent}</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Eșuate
            </CardTitle>
            <TrendingUp className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">{stats.failed}</div>
          </CardContent>
        </Card>
      </div>

      {/* Quick Actions */}
      <div className="grid gap-4 md:grid-cols-2">
        <Link href="/invoices">
          <Card className="hover:bg-accent transition-colors cursor-pointer h-full">
            <CardHeader>
              <CardTitle className="flex items-center gap-3">
                <FileText className="h-5 w-5 text-primary" />
                Facturi
              </CardTitle>
              <CardDescription>
                Vezi toate facturile, trimite sau reîncearcă cele eșuate
              </CardDescription>
            </CardHeader>
          </Card>
        </Link>

        <Link href="/data">
          <Card className="hover:bg-accent transition-colors cursor-pointer h-full">
            <CardHeader>
              <CardTitle className="flex items-center gap-3">
                <Database className="h-5 w-5 text-primary" />
                Date
              </CardTitle>
              <CardDescription>
                Vezi lista de parteneri și produse disponibile
              </CardDescription>
            </CardHeader>
          </Card>
        </Link>
      </div>
    </div>
  );
}
