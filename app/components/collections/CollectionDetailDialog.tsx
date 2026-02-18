"use client";

import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { CollectionStatusBadge } from "./CollectionStatusBadge";
import { toast } from "sonner";
import { useState } from "react";
import { Loader2, Printer, FileText, Calendar, CreditCard, AlertCircle, FileCheck2 } from "lucide-react";
import { format } from "date-fns";
import { ro } from "date-fns/locale";
import type { Collection } from "@/lib/tauri/types";
import { printCollectionToHtml } from "@/lib/tauri/commands";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

interface CollectionDetailDialogProps {
    collection: Collection | null;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function CollectionDetailDialog({
    collection,
    open,
    onOpenChange,
}: CollectionDetailDialogProps) {
    const [isPrinting, setIsPrinting] = useState(false);

    const handlePrint = async () => {
        if (!collection) return;
        setIsPrinting(true);
        try {
            const selectedPrinter = typeof window !== "undefined" ? localStorage.getItem("selectedPrinter") : null;
            await printCollectionToHtml(collection.id, selectedPrinter || undefined);
            toast.success("Chitanța a fost trimisă la imprimantă.");
        } catch (error) {
            console.error("Print collection error:", error);
            toast.error(`Eroare la imprimarea chitanței: ${String(error)}`);
        } finally {
            setIsPrinting(false);
        }
    };

    const formatAmount = (value: number) =>
        new Intl.NumberFormat("ro-RO", { style: "currency", currency: "RON" }).format(value);

    const formatDate = (dateStr?: string) => {
        if (!dateStr) return "-";
        return format(new Date(dateStr), "dd MMMM yyyy", { locale: ro });
    };

    const formatDateTime = (dateStr?: string) => {
        if (!dateStr) return "-";
        return format(new Date(dateStr), "dd.MM.yyyy HH:mm", { locale: ro });
    };

    // Helper to clean up error messages
    const getCleanErrorMessage = (errorMsg: string) => {
        if (!errorMsg) return "";
        // Remove raw JSON arrays like ["..."]
        let clean = errorMsg.replace(/\["|"]/g, "");
        // Remove file paths
        clean = clean.replace(/C:\\Users\\[^\s]*/g, "(vezi loguri)");
        // Remove "API Error: error;" prefix
        clean = clean.replace(/^API Error:\s*error;\s*/i, "");
        return clean.trim();
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-md p-0 overflow-hidden gap-0">
                <DialogHeader className="p-6 pb-2">
                    <DialogTitle className="flex items-center gap-2 text-xl">
                        <FileText className="h-5 w-5 text-muted-foreground" />
                        Detalii Chitanță
                    </DialogTitle>
                </DialogHeader>

                {collection ? (
                    <div className="flex flex-col">
                        <div className="px-6 pb-6 space-y-6">
                            {/* Header Section with Partner & Status */}
                            <div className="flex items-start justify-between gap-4">
                                <div className="space-y-1">
                                    <h3 className="font-semibold text-base leading-tight">
                                        {collection.partner_name || "Partener Necunoscut"}
                                    </h3>
                                    <p className="text-sm text-muted-foreground font-medium">
                                        {collection.receipt_series} {collection.receipt_number}
                                    </p>
                                </div>
                                <CollectionStatusBadge status={collection.status} />
                            </div>

                            <Separator />

                            {/* Details Grid */}
                            <div className="grid grid-cols-2 gap-6">
                                <div className="space-y-1.5">
                                    <span className="text-xs text-muted-foreground flex items-center gap-1.5 font-medium">
                                        <Calendar className="h-3.5 w-3.5" />
                                        Data încasării
                                    </span>
                                    <p className="text-sm font-medium pl-5">
                                        {formatDate(collection.data_incasare)}
                                    </p>
                                </div>
                                <div className="space-y-1.5">
                                    <span className="text-xs text-muted-foreground flex items-center gap-1.5 font-medium">
                                        <CreditCard className="h-3.5 w-3.5" />
                                        Valoare
                                    </span>
                                    <p className="text-lg font-bold pl-5">
                                        {formatAmount(collection.valoare)}
                                    </p>
                                </div>
                            </div>

                            {/* Invoice Info */}
                            <div className="bg-muted/30 rounded-lg border p-4 space-y-3">
                                <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground mb-2">
                                    <FileCheck2 className="h-4 w-4" />
                                    <span>Factura achitată</span>
                                </div>
                                <div className="grid grid-cols-[80px_1fr] gap-x-4 gap-y-2 text-sm">
                                    <span className="text-muted-foreground">Document:</span>
                                    <span className="font-semibold">
                                        {collection.serie_factura} {collection.numar_factura}
                                    </span>

                                    {collection.cod_document && (
                                        <>
                                            <span className="text-muted-foreground">Cod intern:</span>
                                            <span className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded w-fit">
                                                {collection.cod_document}
                                            </span>
                                        </>
                                    )}
                                </div>
                            </div>

                            {/* Status Messages */}
                            {collection.status === "failed" && collection.error_message && (
                                <div className="bg-red-50 dark:bg-red-900/10 p-4 rounded-lg border border-red-100 dark:border-red-900/50 space-y-2">
                                    <div className="flex items-center gap-2 text-red-700 dark:text-red-400 font-medium text-sm">
                                        <AlertCircle className="h-4 w-4" />
                                        Eroare la trimitere
                                    </div>
                                    <p className="text-xs text-red-600 dark:text-red-300 leading-relaxed break-words">
                                        {getCleanErrorMessage(collection.error_message)}
                                    </p>
                                </div>
                            )}

                            {collection.status === "synced" && (
                                <div className="text-center">
                                    <p className="text-xs text-muted-foreground bg-green-50 dark:bg-green-900/10 text-green-700 dark:text-green-400 px-3 py-1.5 rounded-full inline-block border border-green-100 dark:border-green-900/50">
                                        Sincronizat: {formatDateTime(collection.synced_at)}
                                    </p>
                                </div>
                            )}
                        </div>

                        {/* Footer Actions */}
                        <div className="bg-muted/40 p-4 flex justify-end gap-3 border-t">
                            <Button
                                variant="outline"
                                onClick={() => onOpenChange(false)}
                                className="h-9"
                            >
                                Închide
                            </Button>
                            <Button
                                onClick={handlePrint}
                                disabled={isPrinting}
                                className="h-9 gap-2 min-w-[140px]"
                            >
                                {isPrinting ? (
                                    <Loader2 className="h-4 w-4 animate-spin" />
                                ) : (
                                    <Printer className="h-4 w-4" />
                                )}
                                Printează
                            </Button>
                        </div>
                    </div>
                ) : (
                    <div className="p-12 text-center text-muted-foreground">
                        <p>Nu s-au găsit detalii</p>
                    </div>
                )}
            </DialogContent>
        </Dialog>
    );
}
