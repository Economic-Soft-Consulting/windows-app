"use client";

import { useEffect, useMemo, useState } from "react";
import { Loader2, Printer, Receipt, TrendingUp } from "lucide-react";
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
import { Badge } from "@/components/ui/badge";
import { getCollectionsReport, getSalesProductsReport, printReportHtml } from "@/lib/tauri/commands";
import type { CollectionsReportItem, SalesProductReportItem } from "@/lib/tauri/types";
import { formatCurrency } from "@/lib/utils";
import { toast } from "sonner";
import { useAuth } from "@/app/contexts/AuthContext";

function toInputDate(date: Date): string {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function formatCollectionStatusLabel(status: string): string {
  if (status === "synced") return "Sincronizat";
  if (status === "failed") return "Eșuat";
  if (status === "sending") return "Se trimite";
  return "În așteptare";
}

export default function ReportsPage() {
  const { isAdmin } = useAuth();
  const [loadingSales, setLoadingSales] = useState(true);
  const [loadingCollections, setLoadingCollections] = useState(true);
  const [salesRows, setSalesRows] = useState<SalesProductReportItem[]>([]);
  const [collectionsRows, setCollectionsRows] = useState<CollectionsReportItem[]>([]);

  const today = useMemo(() => toInputDate(new Date()), []);

  const [salesStartDate, setSalesStartDate] = useState(today);
  const [salesEndDate, setSalesEndDate] = useState(today);
  const [collectionsStartDate, setCollectionsStartDate] = useState(today);
  const [collectionsEndDate, setCollectionsEndDate] = useState(today);

  useEffect(() => {
    if (!isAdmin) {
      setSalesStartDate(today);
      setSalesEndDate(today);
      setCollectionsStartDate(today);
      setCollectionsEndDate(today);
    }
  }, [isAdmin, today]);

  const printSection = async (sectionId: string, title: string, reportName: string) => {
    const element = document.getElementById(sectionId);
    if (!element) return;

    const htmlDocument = `
      <html>
        <head>
          <title>${title}</title>
          <style>
            body { font-family: Arial, sans-serif; padding: 24px; color: #111; }
            h1 { font-size: 20px; margin-bottom: 16px; }
            table { width: 100%; border-collapse: collapse; margin-top: 12px; }
            th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
            th { background: #f4f4f4; }
            .right { text-align: right; }
          </style>
        </head>
        <body>
          <h1>${title}</h1>
          ${element.innerHTML}
        </body>
      </html>
    `;

    try {
      const selectedPrinter = typeof window !== "undefined"
        ? localStorage.getItem("selectedPrinter")
        : null;
      const path = await printReportHtml(reportName, htmlDocument, selectedPrinter || undefined);
      toast.success(`Raport salvat și trimis la print: ${path}`);
    } catch (error) {
      console.error("Failed to print report html:", error);
      toast.error("Raportul nu a putut fi trimis la print");
    }
  };

  const handlePrintSalesHtml = async () => {
    try {
      const rows = await getSalesProductsReport(salesStartDate || undefined, salesEndDate || undefined);

      const grouped = new Map<string, {
        productName: string;
        productClass: string;
        totalQuantity: number;
        totalCofrage: number;
        totalWithoutVat: number;
        totalWithVat: number;
        lines: SalesProductReportItem[];
      }>();

      rows.forEach((row) => {
        const key = `${row.product_class || "Fără categorie"}::${row.product_name}`;
        const current = grouped.get(key) || {
          productName: row.product_name,
          productClass: row.product_class || "Fără categorie",
          totalQuantity: 0,
          totalCofrage: 0,
          totalWithoutVat: 0,
          totalWithVat: 0,
          lines: [],
        };

        current.totalQuantity += row.total_quantity;
        current.totalCofrage += row.total_cofrage;
        current.totalWithoutVat += row.total_without_vat;
        current.totalWithVat += row.total_with_vat;
        current.lines.push(row);

        grouped.set(key, current);
      });

      const groupedList = Array.from(grouped.values()).sort((a, b) => {
        if (a.productClass !== b.productClass) return a.productClass.localeCompare(b.productClass);
        return a.productName.localeCompare(b.productName);
      });

      const sectionHtml = groupedList
        .map((group) => {
          const linesHtml = group.lines
            .map(
              (line) => {
                const lineVat = line.total_with_vat - line.total_without_vat;
                return `
                <div class="line-item">
                  <div class="line-head">${line.partner_name}</div>
                  <div class="line-meta">Doc ${line.invoice_series} / ${line.invoice_number}</div>
                  <div class="line-values">Cant ${line.total_quantity.toFixed(2)} | Cof ${line.total_cofrage.toFixed(2)} | Preț ${line.total_without_vat.toFixed(2)} | TVA ${lineVat.toFixed(2)} | Preț cu TVA ${line.total_with_vat.toFixed(2)}</div>
                </div>
              `;
              }
            )
            .join("");

          const groupVat = group.totalWithVat - group.totalWithoutVat;

          return `
            <div class="group">
              <h2>${group.productClass} • ${group.productName}</h2>
              <div class="meta meta-total" title="Total produs: Cantitate ${group.totalQuantity.toFixed(2)} | Cofraje ${group.totalCofrage.toFixed(2)} | Preț ${group.totalWithoutVat.toFixed(2)} | TVA ${groupVat.toFixed(2)} | Preț cu TVA ${group.totalWithVat.toFixed(2)}">
                Total: C ${group.totalQuantity.toFixed(2)} | Cf ${group.totalCofrage.toFixed(2)} | P ${group.totalWithoutVat.toFixed(2)} | TVA ${groupVat.toFixed(2)} | P+TVA ${group.totalWithVat.toFixed(2)}
              </div>
              ${linesHtml}
            </div>
          `;
        })
        .join("");

      const grandTotals = rows.reduce(
        (acc, row) => {
          acc.quantity += row.total_quantity;
          acc.cofrage += row.total_cofrage;
          acc.withoutVat += row.total_without_vat;
          acc.withVat += row.total_with_vat;
          return acc;
        },
        { quantity: 0, cofrage: 0, withoutVat: 0, withVat: 0 }
      );

      const intervalLabel = `${salesStartDate || "-"} - ${salesEndDate || "-"}`;

      const htmlDocument = `
        <html>
          <head>
            <title>Raport Vânzări (Print)</title>
            <style>
              @page { size: 80mm 297mm; margin: 2mm; }
              body { font-family: Arial, sans-serif; width: 76mm; margin: 0; padding: 1mm; color: #111; font-size: 10.4px; line-height: 1.24; box-sizing: border-box; }
              h1 { font-size: 15px; margin: 0 0 5px 0; text-align: center; }
              h2 { font-size: 11.4px; margin: 0 0 3px 0; }
              .meta { color: #333; margin-bottom: 4px; font-size: 9.4px; }
              .meta-total { white-space: nowrap; font-size: 8.4px; letter-spacing: -0.08px; overflow: hidden; }
              .group { margin-top: 6px; padding-top: 4px; border-top: 1px dashed #999; }
              .line-item { padding: 4px 0; border-bottom: 1px dotted #ddd; }
              .line-head { font-size: 10.4px; font-weight: 700; word-break: break-word; }
              .line-meta { font-size: 9.3px; color: #333; margin-top: 1px; }
              .line-values { font-size: 9.3px; color: #111; margin-top: 1px; word-break: break-word; }
              .grand-total { margin-top: 9px; padding-top: 6px; border-top: 1px solid #555; font-size: 10.6px; }
            </style>
          </head>
          <body>
            <h1>Raport Vânzări</h1>
            <div class="meta">Interval: ${intervalLabel}</div>
            ${sectionHtml}
            <div class="grand-total">
              <strong>Total general:</strong>
              Cant ${grandTotals.quantity.toFixed(2)} | Cof ${grandTotals.cofrage.toFixed(2)} | Preț ${grandTotals.withoutVat.toFixed(2)} | TVA ${(grandTotals.withVat - grandTotals.withoutVat).toFixed(2)} | Preț cu TVA ${grandTotals.withVat.toFixed(2)}
            </div>
          </body>
        </html>
      `;

      try {
        const selectedPrinter = typeof window !== "undefined"
          ? localStorage.getItem("selectedPrinter")
          : null;
        const path = await printReportHtml("raport_vanzari_detaliat", htmlDocument, selectedPrinter || undefined);
        toast.success(`Raport salvat și trimis la print: ${path}`);
      } catch (error) {
        console.error("Failed to print sales report html:", error);
        toast.error("Raportul de vânzări nu a putut fi trimis la print");
      }
    } catch (error) {
      console.error("Failed to print sales HTML:", error);
      toast.error("Eroare la generarea printului HTML pentru vânzări");
    }
  };

  const handlePrintCollectionsHtml = async () => {
    try {
      const rows = await getCollectionsReport(
        collectionsStartDate || undefined,
        collectionsEndDate || undefined
      );

      const grouped = new Map<string, {
        partnerName: string;
        totalCount: number;
        totalAmount: number;
        lines: CollectionsReportItem[];
      }>();

      rows.forEach((row) => {
        const key = row.partner_name || "Partener necunoscut";
        const current = grouped.get(key) || {
          partnerName: key,
          totalCount: 0,
          totalAmount: 0,
          lines: [],
        };

        current.totalCount += row.collection_count;
        current.totalAmount += row.total_amount;
        current.lines.push(row);
        grouped.set(key, current);
      });

      const groupedList = Array.from(grouped.values()).sort((a, b) =>
        a.partnerName.localeCompare(b.partnerName)
      );

      const sectionHtml = groupedList
        .map((group) => {
          const linesHtml = group.lines
            .sort((a, b) => a.status.localeCompare(b.status))
            .map(
              (line) => `
                <div class="line-item">
                  <div class="line-head">Status: ${formatCollectionStatusLabel(line.status)}</div>
                  <div class="line-values">Chitanțe ${line.collection_count} | Valoare ${line.total_amount.toFixed(2)} RON</div>
                </div>
              `
            )
            .join("");

          return `
            <div class="group">
              <h2>${group.partnerName}</h2>
              <div class="meta">
                Total partener: Chitanțe ${group.totalCount} | Valoare ${group.totalAmount.toFixed(2)} RON
              </div>
              ${linesHtml}
            </div>
          `;
        })
        .join("");

      const totals = rows.reduce(
        (acc, row) => {
          acc.count += row.collection_count;
          acc.amount += row.total_amount;
          if (row.status === "synced") acc.synced += row.total_amount;
          else if (row.status === "failed") acc.failed += row.total_amount;
          else acc.pending += row.total_amount;
          return acc;
        },
        { count: 0, amount: 0, synced: 0, failed: 0, pending: 0 }
      );

      const intervalLabel = `${collectionsStartDate || "-"} - ${collectionsEndDate || "-"}`;

      const htmlDocument = `
        <html>
          <head>
            <title>Raport Încasări (Print)</title>
            <style>
              @page { size: 80mm 297mm; margin: 2mm; }
              body { font-family: Arial, sans-serif; width: 76mm; margin: 0; padding: 1mm; color: #111; font-size: 9px; line-height: 1.2; box-sizing: border-box; }
              h1 { font-size: 13px; margin: 0 0 4px 0; text-align: center; }
              h2 { font-size: 10px; margin: 0 0 2px 0; }
              .meta { color: #333; margin-bottom: 3px; font-size: 8.5px; }
              .group { margin-top: 6px; padding-top: 4px; border-top: 1px dashed #999; }
              .line-item { padding: 3px 0; border-bottom: 1px dotted #ddd; }
              .line-head { font-size: 9px; font-weight: 700; word-break: break-word; }
              .line-values { font-size: 8px; color: #111; margin-top: 1px; word-break: break-word; }
              .grand-total { margin-top: 8px; padding-top: 5px; border-top: 1px solid #555; font-size: 9.5px; }
            </style>
          </head>
          <body>
            <h1>Raport Încasări</h1>
            <div class="meta">Interval: ${intervalLabel}</div>
            ${sectionHtml}
            <div class="grand-total">
              <strong>Total general:</strong><br>
              Chitanțe ${totals.count} | Total ${totals.amount.toFixed(2)} RON<br>
              Sincronizat ${totals.synced.toFixed(2)} | În așteptare ${totals.pending.toFixed(2)} | Eșuat ${totals.failed.toFixed(2)}
            </div>
          </body>
        </html>
      `;

      try {
        const selectedPrinter = typeof window !== "undefined"
          ? localStorage.getItem("selectedPrinter")
          : null;
        const path = await printReportHtml("raport_incasari_detaliat", htmlDocument, selectedPrinter || undefined);
        toast.success(`Raport salvat și trimis la print: ${path}`);
      } catch (error) {
        console.error("Failed to print collections report html:", error);
        toast.error("Raportul de încasări nu a putut fi trimis la print");
      }
    } catch (error) {
      console.error("Failed to print collections HTML:", error);
      toast.error("Eroare la generarea printului HTML pentru încasări");
    }
  };

  const loadSales = async () => {
    setLoadingSales(true);
    try {
      const data = await getSalesProductsReport(salesStartDate || undefined, salesEndDate || undefined);
      setSalesRows(data);
    } catch (error) {
      console.error("Failed to load sales report:", error);
      toast.error("Eroare la încărcarea raportului de vânzări");
    } finally {
      setLoadingSales(false);
    }
  };

  const loadCollections = async () => {
    setLoadingCollections(true);
    try {
      const data = await getCollectionsReport(
        collectionsStartDate || undefined,
        collectionsEndDate || undefined
      );
      setCollectionsRows(data);
    } catch (error) {
      console.error("Failed to load collections report:", error);
      toast.error("Eroare la încărcarea raportului de încasări");
    } finally {
      setLoadingCollections(false);
    }
  };

  useEffect(() => {
    loadSales();
  }, [salesStartDate, salesEndDate]);

  useEffect(() => {
    loadCollections();
  }, [collectionsStartDate, collectionsEndDate]);

  const salesTotals = useMemo(() => {
    return salesRows.reduce(
      (acc, row) => {
        acc.quantity += row.total_quantity;
        acc.amountWithoutVat += row.total_without_vat;
        acc.amountWithVat += row.total_with_vat;
        acc.vat += row.total_with_vat - row.total_without_vat;
        acc.products.add(`${row.product_class || "Fără categorie"}::${row.product_name}`);
        acc.cofrage += row.total_cofrage;
        return acc;
      },
      { amountWithoutVat: 0, amountWithVat: 0, vat: 0, quantity: 0, cofrage: 0, products: new Set<string>() }
    );
  }, [salesRows]);

  const salesProducts = useMemo(() => {
    const grouped = new Map<string, {
      key: string;
      productName: string;
      productClass: string;
      totalQuantity: number;
      totalCofrage: number;
      totalWithoutVat: number;
      totalWithVat: number;
      lines: SalesProductReportItem[];
    }>();

    salesRows.forEach((row) => {
      const key = `${row.product_class || "Fără categorie"}::${row.product_name}`;
      const current = grouped.get(key) || {
        key,
        productName: row.product_name,
        productClass: row.product_class || "Fără categorie",
        totalQuantity: 0,
        totalCofrage: 0,
        totalWithoutVat: 0,
        totalWithVat: 0,
        lines: [],
      };

      current.totalQuantity += row.total_quantity;
      current.totalCofrage += row.total_cofrage;
      current.totalWithoutVat += row.total_without_vat;
      current.totalWithVat += row.total_with_vat;
      current.lines.push(row);

      grouped.set(key, current);
    });

    return Array.from(grouped.values()).sort((a, b) => {
      if (a.productClass !== b.productClass) return a.productClass.localeCompare(b.productClass);
      return a.productName.localeCompare(b.productName);
    });
  }, [salesRows]);

  const collectionsTotals = useMemo(() => {
    return collectionsRows.reduce(
      (acc, row) => {
        acc.amount += row.total_amount;
        acc.count += row.collection_count;
        return acc;
      },
      { amount: 0, count: 0 }
    );
  }, [collectionsRows]);

  return (
    <div className="space-y-4 h-full flex flex-col">
      <div>
        <h1 className="text-2xl font-bold">Rapoarte Centralizatoare</h1>
        <p className="text-muted-foreground">Vânzări și încasări agregate pe partener</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Total Vânzări (cu TVA)</CardTitle>
            <div className="text-2xl font-bold">{formatCurrency(salesTotals.amountWithVat)}</div>
            <CardDescription>
              {salesTotals.products.size} produse • Cantitate {salesTotals.quantity.toFixed(2)} • Cofraje {salesTotals.cofrage.toFixed(2)} • Fără TVA {formatCurrency(salesTotals.amountWithoutVat)} • TVA {formatCurrency(salesTotals.vat)}
            </CardDescription>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Total Încasări</CardTitle>
            <div className="text-2xl font-bold">{formatCurrency(collectionsTotals.amount)}</div>
            <CardDescription>{collectionsTotals.count} chitanțe în perioada selectată</CardDescription>
          </CardHeader>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-2 flex-1 min-h-0">
        <Card className="flex flex-col min-h-0">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5" />
              Raport Vânzări
            </CardTitle>
            <CardDescription>Pe produse: total sus, apoi desfacere per partener/factură</CardDescription>
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
              <Button
                onClick={handlePrintSalesHtml}
                className="h-10 w-full gap-2"
              >
                <Printer className="h-4 w-4" />
                Printează
              </Button>
            </div>
          </CardHeader>
          <CardContent className="flex-1 min-h-0 overflow-auto">
            {loadingSales ? (
              <div className="h-40 flex items-center justify-center">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : salesRows.length === 0 ? (
              <div className="h-40 flex items-center justify-center text-muted-foreground text-sm">
                Fără date de vânzări pentru intervalul selectat
              </div>
            ) : (
              <div id="sales-report-table">
                <div className="space-y-4">
                  {salesProducts.map((product) => {
                    return (
                      <details key={product.key} className="border rounded-md p-3 bg-card" open>
                        <summary className="list-none cursor-pointer">
                          <div className="flex items-center justify-between gap-3">
                            <div>
                              <div className="text-sm font-semibold">{product.productClass} • {product.productName}</div>
                              <div className="text-xs text-muted-foreground mt-0.5">
                                Total: Cantitate {product.totalQuantity.toFixed(2)} • Cofraje {product.totalCofrage.toFixed(2)} • Fără TVA {formatCurrency(product.totalWithoutVat)} • Cu TVA {formatCurrency(product.totalWithVat)}
                              </div>
                            </div>
                            <Badge variant="secondary">{product.lines.length} linii</Badge>
                          </div>
                        </summary>

                        <div className="mt-3">
                          <Table>
                            <TableHeader>
                              <TableRow>
                                <TableHead>Partener</TableHead>
                                <TableHead>Nr Factură/Serie</TableHead>
                                <TableHead className="text-right">Cantitate</TableHead>
                                <TableHead className="text-right">Nr Cofrage</TableHead>
                                <TableHead className="text-right">Preț fără TVA</TableHead>
                                <TableHead className="text-right">Preț cu TVA</TableHead>
                              </TableRow>
                            </TableHeader>
                            <TableBody>
                              {product.lines.map((row) => (
                                <TableRow key={`${product.key}-${row.partner_name}-${row.invoice_series}-${row.invoice_number}-${row.created_at}`}>
                                  <TableCell className="font-medium">{row.partner_name}</TableCell>
                                  <TableCell>{row.invoice_series} / {row.invoice_number}</TableCell>
                                  <TableCell className="text-right">{row.total_quantity.toFixed(2)}</TableCell>
                                  <TableCell className="text-right">{row.total_cofrage.toFixed(2)}</TableCell>
                                  <TableCell className="text-right">{formatCurrency(row.total_without_vat)}</TableCell>
                                  <TableCell className="text-right">{formatCurrency(row.total_with_vat)}</TableCell>
                                </TableRow>
                              ))}
                              <TableRow>
                                <TableCell className="font-semibold">Total</TableCell>
                                <TableCell />
                                <TableCell className="text-right font-semibold">{product.totalQuantity.toFixed(2)}</TableCell>
                                <TableCell className="text-right font-semibold">{product.totalCofrage.toFixed(2)}</TableCell>
                                <TableCell className="text-right font-semibold">{formatCurrency(product.totalWithoutVat)}</TableCell>
                                <TableCell className="text-right font-semibold">{formatCurrency(product.totalWithVat)}</TableCell>
                              </TableRow>
                            </TableBody>
                          </Table>
                        </div>
                      </details>
                    );
                  })}
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="flex flex-col min-h-0">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Receipt className="h-5 w-5" />
              Raport Încasări
            </CardTitle>
            <CardDescription>Centralizator pe partener și status</CardDescription>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 pt-1">
              <Input
                type="date"
                value={collectionsStartDate}
                onChange={(e) => setCollectionsStartDate(e.target.value)}
                disabled={!isAdmin}
                className="h-10"
              />
              <Input
                type="date"
                value={collectionsEndDate}
                onChange={(e) => setCollectionsEndDate(e.target.value)}
                disabled={!isAdmin}
                className="h-10"
              />
              <Button
                onClick={handlePrintCollectionsHtml}
                className="h-10 w-full gap-2"
              >
                <Printer className="h-4 w-4" />
                Printează
              </Button>
            </div>
          </CardHeader>
          <CardContent className="flex-1 min-h-0 overflow-auto">
            {loadingCollections ? (
              <div className="h-40 flex items-center justify-center">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : collectionsRows.length === 0 ? (
              <div className="h-40 flex items-center justify-center text-muted-foreground text-sm">
                Fără date de încasări pentru data selectată
              </div>
            ) : (
              <div id="collections-report-table">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Partener</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead className="text-right">Chitanțe</TableHead>
                    <TableHead className="text-right">Valoare</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {collectionsRows.map((row, index) => (
                    <TableRow key={`${row.partner_name}-${row.status}-${index}`}>
                      <TableCell className="font-medium">{row.partner_name}</TableCell>
                      <TableCell>
                        <Badge variant={row.status === "synced" ? "default" : row.status === "failed" ? "destructive" : "secondary"}>
                          {row.status}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right">{row.collection_count}</TableCell>
                      <TableCell className="text-right">{formatCurrency(row.total_amount)}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
