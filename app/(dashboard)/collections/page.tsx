"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import {
    Plus,
    FileText,
    Loader2,
    LayoutGrid,
    Table as TableIcon,
    MoreHorizontal,
    Send,
    RotateCcw,
    Printer,
    Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card";
import { CollectionStatusBadge } from "@/app/components/collections/CollectionStatusBadge";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { getCollections, sendCollection, printCollectionToHtml, deleteCollection } from "@/lib/tauri/commands";
import type { Collection, CollectionStatus } from "@/lib/tauri/types";
import { toast } from "sonner";
import { format } from "date-fns";
import { ro } from "date-fns/locale";
import { useAuth } from "@/app/contexts/AuthContext";

type TabValue = "all" | CollectionStatus;
type ViewMode = "grid" | "table";

const formatCollectionErrorMessage = (rawError?: string | null): string => {
    if (!rawError || !rawError.trim()) {
        return "A apărut o problemă la trimiterea chitanței.";
    }

    const error = rawError.trim();
    const lower = error.toLowerCase();

    const missingInvoiceMatch = error.match(/nu\s+gasesc\s+in\s+baza\s+de\s+date\s+factura\s+([^\]"'\n]+)/i);
    if (missingInvoiceMatch) {
        const invoiceRef = missingInvoiceMatch[1].trim();
        return `Factura ${invoiceRef} nu există în baza centrală. Trimite mai întâi factura, apoi retrimite chitanța.`;
    }

    if (lower.includes("sumatrapdf not found")) {
        return "Nu s-a găsit aplicația de printare. Instalează SumatraPDF sau verifică setările de imprimare.";
    }

    if (lower.includes("nu am putut salva chitanța local")) {
        return "Chitanța nu s-a putut salva local. Verifică permisiunile de scriere și spațiul disponibil.";
    }

    if (lower.includes("api error")) {
        return "Serverul a respins trimiterea chitanței. Verifică datele documentului și încearcă din nou.";
    }

    if (lower.includes("chitanță salvată:") || lower.includes("chitanta salvata:")) {
        return "Trimiterea a eșuat, dar chitanța a fost salvată local și poate fi printată.";
    }

    return "Trimiterea chitanței a eșuat. Te rugăm să încerci din nou.";
};

export default function CollectionsPage() {
    const [collections, setCollections] = useState<Collection[]>([]);
    const [countsSource, setCountsSource] = useState<Collection[]>([]);
    const [loading, setLoading] = useState(true);
    const [activeTab, setActiveTab] = useState<TabValue>("all");
    const [viewMode, setViewMode] = useState<ViewMode>("grid");
    const [actionId, setActionId] = useState<string | null>(null);
    const [actionType, setActionType] = useState<"send" | "print" | "delete" | null>(null);
    const { isAgent, isAdmin } = useAuth();

    const today = new Date();
    today.setHours(0, 0, 0, 0);

    const applyAgentTodayFilter = (items: Collection[]) => {
        if (!isAgent) return items;

        return items.filter((item) => {
            const d = new Date(item.created_at || item.data_incasare);
            d.setHours(0, 0, 0, 0);
            return d.getTime() === today.getTime();
        });
    };

    useEffect(() => {
        loadData();
    }, [activeTab]);

    // Auto-refresh when there are sending collections
    useEffect(() => {
        const hasSending = collections.some(c => c.status === 'sending');
        if (!hasSending) return;

        const interval = setInterval(() => {
            loadData();
        }, 2000);

        return () => clearInterval(interval);
    }, [collections]);

    const loadData = async () => {
        setLoading(true);
        try {
            const [filteredData, allData] = await Promise.all([
                getCollections(activeTab === "all" ? undefined : activeTab),
                getCollections(undefined),
            ]);
            setCollections(applyAgentTodayFilter(filteredData));
            setCountsSource(applyAgentTodayFilter(allData));
        } catch (error) {
            console.error("Failed to load collections:", error);
            toast.error("Eroare la încărcarea chitanțelor");
        } finally {
            setLoading(false);
        }
    };

    const handleSendCollection = async (collectionId: string) => {
        console.info("[CHITANTE][UI] Send click", { collectionId });
        setActionId(collectionId);
        setActionType("send");
        try {
            const updatedCollection = await sendCollection(collectionId);
            console.info("[CHITANTE][UI] Send result", {
                collectionId,
                status: updatedCollection.status,
                error: updatedCollection.error_message,
            });
            if (updatedCollection.status === "synced") {
                toast.success("Chitanța a fost trimisă cu succes.");
            } else if (updatedCollection.status === "failed") {
                toast.error(formatCollectionErrorMessage(updatedCollection.error_message));
            } else {
                toast.info("Chitanța este în curs de trimitere...");
            }
            await loadData();
        } catch (error) {
            console.error("Send error:", error);
            toast.error(formatCollectionErrorMessage(String(error)));
        } finally {
            console.info("[CHITANTE][UI] Send finished", { collectionId });
            setActionId(null);
            setActionType(null);
        }
    };

    const handlePrintCollection = async (collectionId: string) => {
        console.info("[CHITANTE][UI] Print click", { collectionId });
        setActionId(collectionId);
        setActionType("print");
        try {
            const selectedPrinter = typeof window !== "undefined"
                ? localStorage.getItem("selectedPrinter")
                : null;
            console.info("[CHITANTE][UI] Print request", {
                collectionId,
                selectedPrinter: selectedPrinter || "default",
            });
            await printCollectionToHtml(collectionId, selectedPrinter || undefined);
            toast.success("Chitanța s-a trimis la imprimantă.");
            console.info("[CHITANTE][UI] Print success", { collectionId });
        } catch (error) {
            console.error("Print collection error:", error);
            toast.error(`Eroare la imprimarea chitanței: ${String(error)}`);
        } finally {
            console.info("[CHITANTE][UI] Print finished", { collectionId });
            setActionId(null);
            setActionType(null);
        }
    };

    const handleDeleteCollection = async (collectionId: string) => {
        setActionId(collectionId);
        setActionType("delete");
        try {
            await deleteCollection(collectionId);
            toast.success("Chitanța a fost ștearsă.");
            await loadData();
        } catch (error) {
            console.error("Delete collection error:", error);
            toast.error(`Eroare la ștergerea chitanței: ${String(error)}`);
        } finally {
            setActionId(null);
            setActionType(null);
        }
    };

    const counts = {
        all: countsSource.length,
        pending: countsSource.filter((c) => c.status === "pending").length,
        synced: countsSource.filter((c) => c.status === "synced").length,
        failed: countsSource.filter((c) => c.status === "failed").length,
    };

    const formatAmount = (value: number) =>
        new Intl.NumberFormat("ro-RO", { style: "currency", currency: "RON" }).format(value);

    return (
        <div className="space-y-4 h-full flex flex-col">
            <div>
                <h1 className="text-2xl font-bold">Chitanțe</h1>
                <p className="text-muted-foreground">
                    Gestionează încasările facturilor către parteneri
                </p>
            </div>

            <div className="flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between">
                <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as TabValue)} className="flex-1">
                    <TabsList className="h-14 flex-wrap sm:flex-nowrap">
                        <TabsTrigger value="all" className="h-11 px-3 sm:px-4 gap-1.5 sm:gap-2">
                            Toate
                            <span className="bg-muted text-muted-foreground px-2 py-0.5 rounded-full text-xs">
                                {counts.all}
                            </span>
                        </TabsTrigger>
                        <TabsTrigger value="pending" className="h-11 px-3 sm:px-4 gap-1.5 sm:gap-2">
                            <span className="hidden sm:inline">În așteptare</span>
                            <span className="sm:hidden">Așteptare</span>
                            {counts.pending > 0 && (
                                <span className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400 px-2 py-0.5 rounded-full text-xs">
                                    {counts.pending}
                                </span>
                            )}
                        </TabsTrigger>
                        <TabsTrigger value="synced" className="h-11 px-3 sm:px-4 gap-1.5 sm:gap-2">
                            Sincronizate
                            {counts.synced > 0 && (
                                <span className="bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 px-2 py-0.5 rounded-full text-xs">
                                    {counts.synced}
                                </span>
                            )}
                        </TabsTrigger>
                        <TabsTrigger value="failed" className="h-11 px-3 sm:px-4 gap-1.5 sm:gap-2">
                            Eșuate
                            {counts.failed > 0 && (
                                <span className="bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400 px-2 py-0.5 rounded-full text-xs">
                                    {counts.failed}
                                </span>
                            )}
                        </TabsTrigger>
                    </TabsList>
                </Tabs>

                <div className="flex items-center gap-2 w-full sm:w-auto">
                    <div className="flex items-center border rounded-lg p-1">
                        <Button
                            variant={viewMode === "table" ? "secondary" : "ghost"}
                            size="sm"
                            onClick={() => setViewMode("table")}
                            className="h-9 px-3"
                        >
                            <TableIcon className="h-4 w-4" />
                        </Button>
                        <Button
                            variant={viewMode === "grid" ? "secondary" : "ghost"}
                            size="sm"
                            onClick={() => setViewMode("grid")}
                            className="h-9 px-3"
                        >
                            <LayoutGrid className="h-4 w-4" />
                        </Button>
                    </div>

                    <Link href="/collections/new" className="flex-1 sm:flex-none">
                        <Button size="lg" className="gap-4 h-14 px-6 w-full sm:w-auto">
                            <Plus className="h-6 w-6" />
                            Chitanță nouă
                        </Button>
                    </Link>
                </div>
            </div>

            <div className="flex-1 min-h-0 overflow-auto">
                {loading ? (
                    <div className="flex items-center justify-center py-12">
                        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                ) : collections.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-16 text-center h-full">
                        <FileText className="h-16 w-16 text-muted-foreground/50 mb-4" />
                        <h3 className="text-lg font-medium">Nu există chitanțe</h3>
                        <p className="text-muted-foreground mt-1 mb-6">
                            {activeTab === "all"
                                ? "Creează prima ta chitanță pentru a începe"
                                : `Nu există chitanțe cu statusul "${activeTab}"`}
                        </p>
                        {activeTab === "all" && (
                            <Link href="/collections/new">
                                <Button className="gap-2">
                                    <Plus className="h-4 w-4" />
                                    Creează chitanță
                                </Button>
                            </Link>
                        )}
                    </div>
                ) : viewMode === "table" ? (
                    <div className="border rounded-lg overflow-hidden min-h-full flex flex-col bg-card">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead className="w-[200px]">Partener</TableHead>
                                    <TableHead className="w-[180px]">Document</TableHead>
                                    <TableHead className="w-[120px]">Data</TableHead>
                                    <TableHead className="text-right w-[120px]">Valoare</TableHead>
                                    <TableHead className="w-[120px]">Status</TableHead>
                                    <TableHead className="text-right w-[80px]">Acțiuni</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {collections.map((collection) => (
                                    <TableRow key={collection.id} className="hover:bg-muted/50">
                                        <TableCell className="font-medium">{collection.partner_name || "Nume Partener"}</TableCell>
                                        <TableCell>
                                            <div className="flex flex-col">
                                                <span>{collection.serie_factura} {collection.numar_factura}</span>
                                                <span className="text-xs text-muted-foreground">{collection.cod_document}</span>
                                            </div>
                                        </TableCell>
                                        <TableCell className="text-sm">
                                            {format(new Date(collection.data_incasare), "dd.MM.yyyy", { locale: ro })}
                                        </TableCell>
                                        <TableCell className="text-right font-medium">{formatAmount(collection.valoare)}</TableCell>
                                        <TableCell>
                                            <CollectionStatusBadge status={collection.status} />
                                        </TableCell>
                                        <TableCell className="text-right">
                                            <DropdownMenu>
                                                <DropdownMenuTrigger asChild>
                                                    <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                                                        <MoreHorizontal className="h-4 w-4" />
                                                    </Button>
                                                </DropdownMenuTrigger>
                                                <DropdownMenuContent align="end" side="bottom" sideOffset={5}>
                                                    <DropdownMenuItem
                                                        disabled={actionId === collection.id}
                                                        onSelect={(e) => {
                                                            e.preventDefault();
                                                            handlePrintCollection(collection.id);
                                                        }}
                                                    >
                                                        <Printer className="mr-2 h-4 w-4" />
                                                        Printează
                                                    </DropdownMenuItem>
                                                    {(collection.status === "pending" || collection.status === "failed") && (
                                                        <DropdownMenuItem
                                                            disabled={actionId === collection.id}
                                                            onSelect={(e) => {
                                                                e.preventDefault();
                                                                handleSendCollection(collection.id);
                                                            }}
                                                        >
                                                            {collection.status === "failed" ? (
                                                                <>
                                                                    <RotateCcw className="mr-2 h-4 w-4" />
                                                                    Retrimite
                                                                </>
                                                            ) : (
                                                                <>
                                                                    <Send className="mr-2 h-4 w-4" />
                                                                    Trimite
                                                                </>
                                                            )}
                                                        </DropdownMenuItem>
                                                    )}
                                                    {isAdmin && (collection.status === "pending" || collection.status === "failed") && (
                                                        <>
                                                            <DropdownMenuSeparator />
                                                            <DropdownMenuItem
                                                                className="text-red-600"
                                                                disabled={actionId === collection.id}
                                                                onSelect={(e) => {
                                                                    e.preventDefault();
                                                                    handleDeleteCollection(collection.id);
                                                                }}
                                                            >
                                                                <Trash2 className="mr-2 h-4 w-4" />
                                                                Șterge
                                                            </DropdownMenuItem>
                                                        </>
                                                    )}
                                                </DropdownMenuContent>
                                            </DropdownMenu>
                                        </TableCell>
                                    </TableRow>
                                ))}
                            </TableBody>
                        </Table>
                    </div>
                ) : (
                    <div className="grid gap-3 pb-4 min-h-full content-start" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))" }}>
                        {collections.map((collection) => (
                            <Card key={collection.id} className="flex flex-col text-sm">
                                <CardHeader className="pb-2 pt-3 px-3">
                                    <div className="space-y-2">
                                        <h3 className="font-semibold text-base leading-tight">{collection.partner_name || "Nume Partener"}</h3>
                                        <div className="flex items-center justify-between gap-2">
                                            <div className="text-xs text-muted-foreground min-w-0 flex-1 truncate">
                                                {collection.serie_factura} {collection.numar_factura}
                                            </div>
                                            <CollectionStatusBadge status={collection.status} />
                                        </div>
                                    </div>
                                </CardHeader>
                                <CardContent className="flex-1 pb-2 px-3">
                                    <div className="space-y-2">
                                        <div className="flex items-center justify-between">
                                            <span className="text-xs text-muted-foreground">Valoare</span>
                                            <span className="text-base font-bold">{formatAmount(collection.valoare)}</span>
                                        </div>
                                        <div className="text-xs text-muted-foreground">
                                            {format(new Date(collection.data_incasare), "dd MMM yyyy", { locale: ro })}
                                        </div>
                                        {collection.status === "failed" && collection.error_message && (
                                            <div
                                                className="text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-1.5 rounded leading-tight"
                                                title={collection.error_message}
                                            >
                                                {formatCollectionErrorMessage(collection.error_message)}
                                            </div>
                                        )}
                                        {collection.status === "synced" && collection.synced_at && (
                                            <div className="text-xs text-green-600 dark:text-green-400">
                                                Trimisă: {format(new Date(collection.synced_at), "dd.MM.yyyy HH:mm", { locale: ro })}
                                            </div>
                                        )}
                                    </div>
                                </CardContent>
                                <CardFooter className="pt-2 px-3 pb-3 border-t gap-2 justify-center flex-wrap">
                                    <Button
                                        variant="outline"
                                        className="h-9 px-3 text-xs"
                                        disabled={actionId === collection.id}
                                        onClick={() => handlePrintCollection(collection.id)}
                                    >
                                        {actionId === collection.id && actionType === "print" ? (
                                            <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                                        ) : (
                                            <Printer className="h-3.5 w-3.5 mr-1" />
                                        )}
                                        Printează
                                    </Button>
                                    {(collection.status === "pending" || collection.status === "failed") && (
                                        <Button
                                            variant={collection.status === "failed" ? "outline" : "default"}
                                            className="h-9 px-3 text-xs"
                                            disabled={actionId === collection.id}
                                            onClick={() => handleSendCollection(collection.id)}
                                        >
                                            {actionId === collection.id && actionType === "send" ? (
                                                <>
                                                    <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                                                    Se trimite...
                                                </>
                                            ) : collection.status === "failed" ? (
                                                <>
                                                    <RotateCcw className="h-3.5 w-3.5 mr-1" />
                                                    Retrimite
                                                </>
                                            ) : (
                                                <>
                                                    <Send className="h-3.5 w-3.5 mr-1" />
                                                    Trimite
                                                </>
                                            )}
                                        </Button>
                                    )}
                                    {isAdmin && (collection.status === "pending" || collection.status === "failed") && (
                                        <Button
                                            variant="ghost"
                                            className="h-9 w-9 p-0 flex-shrink-0 text-red-600 hover:text-red-700 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
                                            disabled={actionId === collection.id}
                                            onClick={() => handleDeleteCollection(collection.id)}
                                            title="Șterge chitanța"
                                        >
                                            {actionId === collection.id && actionType === "delete" ? (
                                                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                                            ) : (
                                                <Trash2 className="h-3.5 w-3.5" />
                                            )}
                                        </Button>
                                    )}
                                </CardFooter>
                            </Card>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}
