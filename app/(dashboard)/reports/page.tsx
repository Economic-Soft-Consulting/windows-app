"use client";

import { useEffect, useMemo, useState } from "react";
import { Loader2, Printer, TrendingUp } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { getDailyCollectionsReport, getSalesPrintReport, printReportHtml } from "@/lib/tauri/commands";
import type { DailyCollectionsReport, SalesPrintItem } from "@/lib/tauri/types";
import { formatCurrency } from "@/lib/utils";
import { toast } from "sonner";
import { useAuth } from "@/app/contexts/AuthContext";

function toInputDate(date: Date): string {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export default function ReportsPage() {
  const { isAdmin } = useAuth();
  const [loadingSales, setLoadingSales] = useState(true);
  const [loadingCollections, setLoadingCollections] = useState(true);
  const [salesRows, setSalesRows] = useState<SalesPrintItem[]>([]);
  const [collectionsReport, setCollectionsReport] = useState<DailyCollectionsReport | null>(null);

  const today = useMemo(() => toInputDate(new Date()), []);

  const [salesStartDate, setSalesStartDate] = useState(today);
  const [salesEndDate, setSalesEndDate] = useState(today);

  useEffect(() => {
    if (!isAdmin) {
      setSalesStartDate(today);
      setSalesEndDate(today);
    }
  }, [isAdmin, today]);

  const loadSales = async () => {
    setLoadingSales(true);
    try {
      const data = await getSalesPrintReport(salesStartDate || undefined, salesEndDate || undefined);
      setSalesRows(data);
    } catch (error) {
      console.error("Failed to load sales report:", error);
      toast.error("Eroare la încărcarea raportului de vânzări");
    } finally {
      setLoadingSales(false);
    }
  };

  useEffect(() => {
    loadSales();
  }, [salesStartDate, salesEndDate]);

  useEffect(() => {
    const loadCollections = async () => {
      setLoadingCollections(true);
      try {
        const reportDate = salesEndDate || salesStartDate;
        const data = await getDailyCollectionsReport(reportDate || undefined);
        setCollectionsReport(data);
      } catch (error) {
        console.error("Failed to load collections for centralizator:", error);
        toast.error("Eroare la încărcarea chitanțelor în centralizator");
      } finally {
        setLoadingCollections(false);
      }
    };

    loadCollections();
  }, [salesStartDate, salesEndDate]);

  const salesByPartner = useMemo(() => {
    const grouped = new Map<string, {
      partner_name: string;
      total_quantity: number;
      total_cofrage: number;
      total_without_vat: number;
      total_with_vat: number;
    }>();

    for (const row of salesRows) {
      const key = row.partner_name;
      const current = grouped.get(key) ?? {
        partner_name: row.partner_name,
        total_quantity: 0,
        total_cofrage: 0,
        total_without_vat: 0,
        total_with_vat: 0,
      };

      current.total_quantity += row.total_quantity;
      current.total_cofrage += row.total_cofrage;
      current.total_without_vat += row.total_without_vat;
      current.total_with_vat += row.total_with_vat;

      grouped.set(key, current);
    }

    return Array.from(grouped.values()).sort((a, b) => a.partner_name.localeCompare(b.partner_name));
  }, [salesRows]);

  const salesTotals = useMemo(() => {
    return salesRows.reduce(
      (acc, row) => {
        acc.cofrage += row.total_cofrage;
        acc.withoutVat += row.total_without_vat;
        acc.withVat += row.total_with_vat;
        acc.quantity += row.total_quantity;
        return acc;
      },
      { quantity: 0, cofrage: 0, withoutVat: 0, withVat: 0 }
    );
  }, [salesRows]);

  const renderSalesRows = () => {
    return salesByPartner.map((row) => (
      <TableRow key={row.partner_name}>
        <TableCell className="font-medium">{row.partner_name.slice(0, 7)}</TableCell>
        <TableCell className="text-right">{row.total_quantity.toFixed(0)}</TableCell>
        <TableCell className="text-right">{row.total_cofrage.toFixed(2)}</TableCell>
        <TableCell className="text-right">{formatCurrency(row.total_without_vat)}</TableCell>
        <TableCell className="text-right">{formatCurrency(row.total_with_vat)}</TableCell>
      </TableRow>
    ));
  };

  const receiptsTotals = useMemo(() => {
    const report = collectionsReport;
    const totalDay = report?.items.reduce((acc, item) => acc + item.amount_from_today_sales, 0) ?? 0;
    const totalDebt = report?.items.reduce((acc, item) => acc + item.amount_from_previous_debt, 0) ?? 0;
    return {
      totalDay,
      totalDebt,
      grandTotal: totalDay + totalDebt,
    };
  }, [collectionsReport]);

  const printSales = async () => {
    const reportDate = salesEndDate || salesStartDate;
    const collections = await getDailyCollectionsReport(reportDate || undefined);

    const formatPrintNumber = (value: number, decimals = 2) =>
      value.toLocaleString("ro-RO", {
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
      });

    const invoiceRows = salesByPartner
      .map((row) => `
        <tr>
          <td class="partner-code">${row.partner_name.slice(0, 7)}</td>
          <td class="right num">${formatPrintNumber(row.total_quantity, 0)}</td>
          <td class="right num">${formatPrintNumber(row.total_cofrage, 2)}</td>
          <td class="right num">${formatPrintNumber(row.total_without_vat, 2)}</td>
          <td class="right num">${formatPrintNumber(row.total_with_vat, 2)}</td>
        </tr>
      `)
      .join("");

    const invoicesTotalQty = salesByPartner.reduce((acc, row) => acc + row.total_quantity, 0);
    const invoicesTotalCofrage = salesByPartner.reduce((acc, row) => acc + row.total_cofrage, 0);
    const invoicesTotalWithoutVat = salesByPartner.reduce((acc, row) => acc + row.total_without_vat, 0);
    const invoicesTotalValue = salesByPartner.reduce((acc, row) => acc + row.total_with_vat, 0);

    const receiptsRows = collections.items
      .map((row) => `
        <tr>
          <td class="partner-code">${row.partner_name.slice(0, 7)}</td>
          <td class="right num">${formatPrintNumber(row.total_amount, 2)}</td>
        </tr>
      `)
      .join("");

    const receiptsTotalDay = collections.items.reduce(
      (acc, item) => acc + item.amount_from_today_sales,
      0
    );
    const receiptsTotalDebt = collections.items.reduce(
      (acc, item) => acc + item.amount_from_previous_debt,
      0
    );
    const receiptsGrandTotal = receiptsTotalDay + receiptsTotalDebt;

    const htmlDocument = `
      <html>
        <head>
          <title>Centralizator zi</title>
          <style>
            @media print {
              @page {
                size: 80mm 297mm;
                margin: 3mm 6mm 3mm 0.5mm;
              }
              body { margin: 0; padding: 0; }
            }
            html { height: 100%; }
            body {
              font-family: Arial, Helvetica, sans-serif;
              width: 68mm;
              margin: 0 auto;
              padding: 2mm;
              font-size: 14px;
              font-weight: bold;
              color: #000;
              line-height: 1.15;
              box-sizing: border-box;
            }
            h1 {
              font-size: 22px;
              text-align: center;
              margin: 0 0 8px 0;
              border-bottom: 2px solid #000;
              text-transform: uppercase;
              padding-bottom: 4px;
            }
            .section {
              margin-top: 8px;
              padding-bottom: 4px;
            }
            h2 {
              font-size: 16px;
              margin: 0 0 4px;
              text-decoration: underline;
            }
            table {
              width: 100%;
              border-collapse: collapse;
              margin-top: 3px;
              font-size: 10.5px;
              table-layout: fixed;
            }
            th, td {
              padding: 5px 3px;
              text-align: left;
              vertical-align: middle;
              box-sizing: border-box;
              overflow: visible;
            }
            th {
              border-bottom: 1px solid #000;
              font-size: 12px;
              line-height: 1.2;
              white-space: normal;
              word-break: break-word;
              padding-bottom: 4px;
            }
            .right { text-align: right; }
            .num {
              white-space: nowrap;
              font-variant-numeric: tabular-nums;
              font-size: 0.9em;
            }
            .invoice-table th:first-child,
            .invoice-table td:first-child,
            .receipts-table th:first-child,
            .receipts-table td:first-child {
              text-align: left;
              font-size: 11px;
            }
            .partner-code {
              white-space: nowrap;
              font-size: 11px;
              letter-spacing: 0.2px;
              text-overflow: unset;
            }
            .invoice-table th:not(:first-child),
            .invoice-table td:not(:first-child),
            .receipts-table th:nth-child(2),
            .receipts-table td:nth-child(2) {
              text-align: right;
            }
            .invoice-table th:nth-child(1), .invoice-table td:nth-child(1) { width: 20%; }
            .invoice-table th:nth-child(2), .invoice-table td:nth-child(2) { width: 20%; }
            .invoice-table th:nth-child(3), .invoice-table td:nth-child(3) { width: 20%; }
            .invoice-table th:nth-child(4), .invoice-table td:nth-child(4) { width: 20%; }
            .invoice-table th:nth-child(5), .invoice-table td:nth-child(5) { width: 20%; }
            .receipts-table th:nth-child(1), .receipts-table td:nth-child(1) { width: 50%; }
            .receipts-table th:nth-child(2), .receipts-table td:nth-child(2) { width: 50%; }
            .totals {
              margin-top: 6px;
              border-top: 1px solid #000;
              padding-top: 4px;
              font-size: 11px;
            }
            .totals strong {
              font-size: 13px;
            }
            .total-facturi-label {
              font-size: 13px;
            }
          </style>
        </head>
        <body>
          <h1>Centralizator zi</h1>

          <div class="section">
            <h2>Facturi eliberate</h2>
            <table class="invoice-table">
              <thead>
                <tr>
                  <th>Cod ext</th>
                  <th class="right">Ouă</th>
                  <th class="right">Cof</th>
                  <th class="right">Fără TVA</th>
                  <th class="right">Cu TVA</th>
                </tr>
              </thead>
              <tbody>${invoiceRows || "<tr><td colspan=\"5\">Fără date</td></tr>"}</tbody>
            </table>
            <div class="totals">
              <strong class="total-facturi-label">Total facturi:</strong>
              Ouă ${formatPrintNumber(invoicesTotalQty, 0)} |
              Cofraje ${formatPrintNumber(invoicesTotalCofrage, 2)} |
              Fără TVA ${formatPrintNumber(invoicesTotalWithoutVat, 2)} |
              Cu TVA ${formatPrintNumber(invoicesTotalValue, 2)}
            </div>
          </div>

          <div class="section">
            <h2>Chitanțe</h2>
            <table class="receipts-table">
              <thead>
                <tr>
                  <th>Cod ext</th>
                  <th class="right">Încasat</th>
                </tr>
              </thead>
              <tbody>${receiptsRows || "<tr><td colspan=\"2\">Fără date</td></tr>"}</tbody>
            </table>
            <div class="totals">
              <strong>Total chitanțe:</strong>
              Total chitanțe pe zi ${collections.receipts_today_invoices_count} |
              Total chitanțe pe solduri ${collections.receipts_previous_debt_count} |
              Total încasat pe zi ${formatPrintNumber(receiptsTotalDay, 2)} |
              Total din solduri ${formatPrintNumber(receiptsTotalDebt, 2)} |
              Total general ${formatPrintNumber(receiptsGrandTotal, 2)}
            </div>
          </div>
        </body>
      </html>
    `;

    try {
      const selectedPrinter = typeof window !== "undefined"
        ? localStorage.getItem("selectedPrinter")
        : null;
      const path = await printReportHtml("centralizator_zi_vanzari", htmlDocument, selectedPrinter || undefined);
      toast.success(`Raport salvat și trimis la print: ${path}`);
    } catch (error) {
      console.error("Failed to print sales report html:", error);
      toast.error("Raportul de vânzări nu a putut fi trimis la print");
    }
  };

  return (
    <div className="space-y-4 h-full flex flex-col">
      <div>
        <h1 className="text-2xl font-bold">Centralizator zi</h1>
        <p className="text-muted-foreground">Vânzări agregate pe partener</p>
      </div>

      <div className="grid gap-4 md:grid-cols-1">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Centralizator zi (cu TVA)</CardTitle>
            <div className="text-2xl font-bold">{formatCurrency(salesTotals.withVat)}</div>
            <CardDescription>
              Ouă {salesTotals.quantity.toFixed(0)} • Cofraje {salesTotals.cofrage.toFixed(2)} • Fără TVA {formatCurrency(salesTotals.withoutVat)} • Cu TVA {formatCurrency(salesTotals.withVat)}
            </CardDescription>
          </CardHeader>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-1 flex-1 min-h-0">
        <Card className="flex flex-col min-h-0">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5" />
              Centralizator zi
            </CardTitle>
            <CardDescription>Facturi eliberate și chitanțe, agregat pe partener</CardDescription>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 pt-1">
              <Input
                type="date"
                value={salesStartDate}
                onChange={(e) => setSalesStartDate(e.target.value)}
                disabled={!isAdmin}
                className="h-10"
              />
              <Input
                type="date"
                value={salesEndDate}
                onChange={(e) => setSalesEndDate(e.target.value)}
                disabled={!isAdmin}
                className="h-10"
              />
              <Button onClick={printSales} className="h-10 w-full gap-2">
                <Printer className="h-4 w-4" />
                Printează
              </Button>
            </div>
          </CardHeader>
          <CardContent className="flex-1 min-h-0 overflow-auto space-y-4">
            {loadingSales ? (
              <div className="h-40 flex items-center justify-center">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : salesByPartner.length === 0 ? (
              <div className="h-40 flex items-center justify-center text-muted-foreground text-sm">
                Fără date de vânzări pentru intervalul selectat
              </div>
            ) : (
              <>
                <div className="rounded-md border">
                  <div className="flex items-center justify-between p-3 border-b bg-muted/30">
                    <span className="font-semibold text-sm">Facturi eliberate</span>
                    <span className="text-xs text-muted-foreground">{salesByPartner.length} parteneri</span>
                  </div>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Cod ext</TableHead>
                        <TableHead className="text-right">Nr ouă</TableHead>
                        <TableHead className="text-right">Nr Cofraje</TableHead>
                        <TableHead className="text-right">Preț fără TVA</TableHead>
                        <TableHead className="text-right">Preț cu TVA</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {renderSalesRows()}
                      <TableRow>
                        <TableCell className="font-semibold">TOTAL</TableCell>
                        <TableCell className="text-right font-semibold">{salesTotals.quantity.toFixed(0)}</TableCell>
                        <TableCell className="text-right font-semibold">{salesTotals.cofrage.toFixed(2)}</TableCell>
                        <TableCell className="text-right font-semibold">{formatCurrency(salesTotals.withoutVat)}</TableCell>
                        <TableCell className="text-right font-semibold">{formatCurrency(salesTotals.withVat)}</TableCell>
                      </TableRow>
                    </TableBody>
                  </Table>
                </div>

                <div className="rounded-md border">
                  <div className="flex items-center justify-between p-3 border-b bg-muted/30">
                    <span className="font-semibold text-sm">Chitanțe</span>
                    <span className="text-xs text-muted-foreground">{collectionsReport?.items.length ?? 0} parteneri</span>
                  </div>
                  {loadingCollections ? (
                    <div className="h-24 flex items-center justify-center">
                      <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                    </div>
                  ) : !collectionsReport || collectionsReport.items.length === 0 ? (
                    <div className="p-4 text-sm text-muted-foreground">Fără chitanțe pentru data selectată</div>
                  ) : (
                    <>
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead>Cod ext</TableHead>
                            <TableHead className="text-right">Încasat</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {collectionsReport.items.map((row) => (
                            <TableRow key={row.partner_name}>
                              <TableCell className="font-medium">{row.partner_name.slice(0, 7)}</TableCell>
                              <TableCell className="text-right">{formatCurrency(row.total_amount)}</TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                      <div className="p-3 border-t bg-muted/20 text-sm space-y-1">
                        <div><span className="font-medium">Total chitanțe pe zi:</span> {collectionsReport.receipts_today_invoices_count}</div>
                        <div><span className="font-medium">Total chitanțe pe solduri:</span> {collectionsReport.receipts_previous_debt_count}</div>
                        <div><span className="font-medium">Total încasat pe zi (facturi eliberate azi):</span> {formatCurrency(receiptsTotals.totalDay)}</div>
                        <div><span className="font-medium">Total din solduri:</span> {formatCurrency(receiptsTotals.totalDebt)}</div>
                        <div><span className="font-semibold">Total general:</span> {formatCurrency(receiptsTotals.grandTotal)}</div>
                      </div>
                    </>
                  )}
                </div>
              </>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
