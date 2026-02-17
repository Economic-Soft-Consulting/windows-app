"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import {
    ArrowLeft,
    ArrowRight,
    Search,
    Building2,
    MapPin,
    CreditCard,
    CheckCircle2,
    FileText,
    Check,
    Loader2
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { format } from "date-fns";
import { ro } from "date-fns/locale";
import {
    getPartners,
    getClientBalances,
    recordCollectionGroup,
    printCollectionToHtml,
    sendCollection
} from "@/lib/tauri/commands";
import type { PartnerWithLocations, ClientBalance, CreateCollectionGroupRequest } from "@/lib/tauri/types";
import { toast } from "sonner";
import { cn } from "@/lib/utils";

export default function NewCollectionPage() {
    const router = useRouter();
    const [step, setStep] = useState<"partner" | "invoice" | "details">("partner");

    // Data
    const [partners, setPartners] = useState<PartnerWithLocations[]>([]);
    const [filteredPartners, setFilteredPartners] = useState<PartnerWithLocations[]>([]);
    const [balances, setBalances] = useState<ClientBalance[]>([]);

    // Selection
    const [selectedPartner, setSelectedPartner] = useState<PartnerWithLocations | null>(null);
    const [selectedBalanceKeys, setSelectedBalanceKeys] = useState<string[]>([]);
    const [selectedBalances, setSelectedBalances] = useState<ClientBalance[]>([]);
    const [allocatedAmounts, setAllocatedAmounts] = useState<Record<string, string>>({});

    // Loading
    const [loadingPartners, setLoadingPartners] = useState(true);
    const [loadingBalances, setLoadingBalances] = useState(false);
    const [saving, setSaving] = useState(false);
    const [search, setSearch] = useState("");

    const steps = [
        { key: "partner" as const, title: "Partener", icon: Building2 },
        { key: "invoice" as const, title: "Factură", icon: FileText },
        { key: "details" as const, title: "Detalii", icon: CreditCard },
    ];

    useEffect(() => {
        loadPartners();
    }, []);

    useEffect(() => {
        if (search) {
            setFilteredPartners(partners.filter(p =>
                p.name.toLowerCase().includes(search.toLowerCase()) ||
                p.cif?.includes(search)
            ));
        } else {
            setFilteredPartners(partners);
        }
    }, [search, partners]);

    const loadPartners = async () => {
        try {
            const data = await getPartners();
            setPartners(data);
            setFilteredPartners(data);
        } catch (error) {
            console.error("Failed to load partners:", error);
            toast.error("Eroare la încărcarea partenerilor");
        } finally {
            setLoadingPartners(false);
        }
    };

    const handlePartnerSelect = async (partner: PartnerWithLocations) => {
        setSelectedPartner(partner);
        setSelectedBalanceKeys([]);
        setSelectedBalances([]);
        setAllocatedAmounts({});
    };

    const getBalanceKey = (balance: ClientBalance) =>
        `${balance.id_partener || ""}|${balance.serie || ""}|${balance.numar || ""}|${balance.cod_document || ""}|${balance.data || ""}`;

    const handleInvoiceSelect = (balance: ClientBalance) => {
        const key = getBalanceKey(balance);
        const isSelected = selectedBalanceKeys.includes(key);

        if (isSelected) {
            setSelectedBalanceKeys((prev) => prev.filter((k) => k !== key));
            setSelectedBalances((prev) => prev.filter((item) => getBalanceKey(item) !== key));
            setAllocatedAmounts((prev) => {
                const next = { ...prev };
                delete next[key];
                return next;
            });
            return;
        }

        setSelectedBalanceKeys((prev) => [...prev, key]);
        setSelectedBalances((prev) => [...prev, balance]);
        setAllocatedAmounts((prev) => ({
            ...prev,
            [key]: (balance.rest || 0).toFixed(2),
        }));
    };

    const parseAllocated = (key: string) => {
        const raw = (allocatedAmounts[key] || "").replace(",", ".");
        const value = Number.parseFloat(raw);
        return Number.isFinite(value) ? value : 0;
    };

    const isAllocationValid = (balance: ClientBalance) => {
        const key = getBalanceKey(balance);
        const value = parseAllocated(key);
        const rest = balance.rest || 0;
        return value > 0 && value <= rest;
    };

    const canGoNext = () => {
        if (step === "partner") return selectedPartner !== null;
        if (step === "invoice") return selectedBalances.length > 0;
        if (step === "details") return selectedBalances.length > 0 && selectedBalances.every(isAllocationValid);
        return false;
    };

    const handleNext = async () => {
        if (step === "partner" && selectedPartner) {
            setStep("invoice");
            return;
        }

        if (step === "invoice" && selectedBalances.length > 0) {
            setStep("details");
            return;
        }

        if (step === "details") {
            await handleSave();
        }
    };

    const handleBack = () => {
        if (step === "details") {
            setStep("invoice");
            return;
        }
        if (step === "invoice") {
            setStep("partner");
            setSelectedBalanceKeys([]);
            setSelectedBalances([]);
            setAllocatedAmounts({});
            return;
        }
        router.push("/collections");
    };

    useEffect(() => {
        const loadBalances = async () => {
            if (step !== "invoice" || !selectedPartner) return;

            setLoadingBalances(true);
            try {
                const data = await getClientBalances(selectedPartner.id);
                setBalances(data);
            } catch (error) {
                console.error("Failed to load balances:", error);
                toast.error("Eroare la încărcarea soldurilor");
            } finally {
                setLoadingBalances(false);
            }
        };

        loadBalances();
    }, [step, selectedPartner]);

    const handleSave = async () => {
        if (!selectedPartner || selectedBalances.length === 0) return;

        const allocations = selectedBalances
            .map((balance) => {
                const key = getBalanceKey(balance);
                const value = parseAllocated(key);
                return {
                    serie_factura: balance.serie,
                    numar_factura: balance.numar,
                    cod_document: balance.cod_document,
                    valoare: value,
                };
            })
            .filter((item) => item.valoare > 0);

        if (allocations.length === 0) {
            toast.error("Introdu valori valide pentru cel puțin o factură.");
            return;
        }

        setSaving(true);
        try {
            const request: CreateCollectionGroupRequest = {
                id_partener: selectedPartner.id,
                partner_name: selectedPartner.name,
                allocations,
            };

            const collectionId = await recordCollectionGroup(request);
            toast.success("Chitanță salvată cu succes");

            try {
                const selectedPrinter = typeof window !== "undefined"
                    ? localStorage.getItem("selectedPrinter")
                    : null;
                await printCollectionToHtml(collectionId, selectedPrinter || undefined);
                toast.success("Chitanța a fost trimisă la imprimantă.");
            } catch (printError) {
                console.error("Auto-print collection failed:", printError);
                toast.warning("Chitanța a fost salvată, dar printarea automată a eșuat.");
            }

            try {
                const sentCollection = await sendCollection(collectionId);
                if (sentCollection.status === "synced") {
                    toast.success("Chitanța a fost trimisă automat.");
                } else if (sentCollection.status === "failed") {
                    toast.warning(sentCollection.error_message || "Chitanța a fost salvată, dar trimiterea automată a eșuat.");
                } else {
                    toast.info("Chitanța este în curs de trimitere automată...");
                }
            } catch (sendError) {
                console.error("Auto-send collection failed:", sendError);
                toast.warning("Chitanța a fost salvată, dar nu s-a putut trimite automat.");
            }

            router.push("/collections");
        } catch (error) {
            console.error("Failed to save collection:", error);
            toast.error("Eroare la salvarea chitanței: " + error);
        } finally {
            setSaving(false);
        }
    };

    return (
        <div className="space-y-6 h-full min-h-0 flex flex-col">
            <div className="flex items-center gap-4">
                <Link href="/collections">
                    <Button variant="ghost" size="icon" className="h-10 w-10">
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                </Link>
                <div>
                    <h1 className="text-2xl font-bold">Chitanță nouă</h1>
                    <p className="text-muted-foreground">
                        Creează o nouă chitanță pentru un partener
                    </p>
                </div>
            </div>

            <div className="max-w-6xl mx-auto flex-1 min-h-0 flex flex-col space-y-4 w-full pb-4">

            <div className="flex items-center justify-between gap-4">
                <Button
                    variant="outline"
                    size="lg"
                    onClick={handleBack}
                    className="gap-2 h-11 px-4"
                >
                    <ArrowLeft className="h-4 w-4" />
                    Înapoi
                </Button>

                <div className="flex items-center gap-2">
                    {steps.map((item, index) => {
                        const isActive = step === item.key;
                        const isCompleted = steps.findIndex((s) => s.key === step) > index;
                        const Icon = item.icon;

                        return (
                            <div key={item.key} className="flex items-center">
                                <div
                                    className={cn(
                                        "flex items-center gap-2 px-4 py-2 rounded-full transition-colors",
                                        isActive && "bg-primary text-primary-foreground",
                                        isCompleted && "bg-primary/20 text-primary",
                                        !isActive && !isCompleted && "bg-muted text-muted-foreground"
                                    )}
                                >
                                    {isCompleted ? <CheckCircle2 className="h-4 w-4" /> : <Icon className="h-4 w-4" />}
                                    <span className="text-sm font-medium">{item.title}</span>
                                </div>
                                {index < steps.length - 1 && (
                                    <div className={cn("w-8 h-0.5 mx-1", isCompleted ? "bg-primary" : "bg-muted")} />
                                )}
                            </div>
                        );
                    })}
                </div>

                <Button
                    size="lg"
                    onClick={handleNext}
                    disabled={!canGoNext() || saving}
                    className="gap-2 h-11 px-4"
                >
                    {step === "details" ? "Salvează" : "Continuă"}
                    {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : <ArrowRight className="h-4 w-4" />}
                </Button>
            </div>

            {step === "partner" && (
                <Card className="flex flex-col">
                    <CardHeader className="pb-3 space-y-2">
                        <div className="relative">
                            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
                            <Input
                                placeholder="Caută partener..."
                                className="pl-9 h-9 text-sm"
                                value={search}
                                onChange={(e) => setSearch(e.target.value)}
                                autoFocus
                            />
                        </div>
                    </CardHeader>
                    <CardContent className="p-0">
                        {loadingPartners ? (
                            <div className="flex items-center justify-center py-12">
                                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                            </div>
                        ) : filteredPartners.length === 0 ? (
                            <div className="flex flex-col items-center justify-center py-12 text-center">
                                <Building2 className="h-12 w-12 text-muted-foreground/50 mb-4" />
                                <h3 className="text-lg font-medium">Nu s-au găsit parteneri</h3>
                                <p className="text-muted-foreground mt-1">Încearcă cu alt termen de căutare</p>
                            </div>
                        ) : (
                            <ScrollArea className="h-[calc(100vh-280px)] min-h-[300px] max-h-[600px] pr-4">
                                <div className="grid gap-2 grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
                                    {filteredPartners.map((partner) => (
                                        <Card
                                            key={partner.id}
                                            className={cn(
                                                "cursor-pointer transition-all hover:border-primary/50 min-h-[68px]",
                                                selectedPartner?.id === partner.id && "border-primary bg-primary/5 ring-2 ring-primary"
                                            )}
                                            onClick={() => handlePartnerSelect(partner)}
                                        >
                                            <CardHeader className="pb-1 pt-1.5 px-2">
                                                <div className="flex items-start justify-between gap-1.5">
                                                    <CardTitle className="text-sm leading-tight line-clamp-2">{partner.name}</CardTitle>
                                                    {selectedPartner?.id === partner.id && (
                                                        <div className="h-5 w-5 rounded-full bg-primary flex items-center justify-center flex-shrink-0">
                                                            <Check className="h-3.5 w-3.5 text-primary-foreground" />
                                                        </div>
                                                    )}
                                                </div>
                                            </CardHeader>
                                            <CardContent className="px-2 pb-1.5">
                                                <div className="space-y-0.5">
                                                    <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
                                                        <MapPin className="h-4 w-4" />
                                                        {partner.locations.length} loc.
                                                    </div>
                                                    <div className="flex items-center gap-1.5 text-sm">
                                                        <span className="text-muted-foreground">Scad:</span>
                                                        <span className="font-medium text-primary">
                                                            {partner.scadenta_la_vanzare ? `${partner.scadenta_la_vanzare}z` : '-'}
                                                        </span>
                                                    </div>
                                                </div>
                                            </CardContent>
                                        </Card>
                                    ))}
                                </div>
                            </ScrollArea>
                        )}
                    </CardContent>
                </Card>
            )}

            {step === "invoice" && selectedPartner && (
                <Card className="flex-1 flex flex-col min-h-0">
                    <CardHeader className="pb-3 border-b bg-muted/20">
                        <div>
                            <CardTitle>{selectedPartner.name}</CardTitle>
                            <CardDescription>Selectează factura pentru plată</CardDescription>
                        </div>
                    </CardHeader>
                    <CardContent className="flex-1 overflow-auto p-0 pb-6">
                        {loadingBalances ? (
                            <div className="flex items-center justify-center h-40">
                                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                            </div>
                        ) : balances.length === 0 ? (
                            <div className="flex flex-col items-center justify-center h-40 text-muted-foreground">
                                <FileText className="h-8 w-8 mb-2 opacity-50" />
                                <p>Nu există facturi cu sold</p>
                            </div>
                        ) : (
                            <div className="grid gap-3 p-4 pb-20 grid-cols-1 md:grid-cols-3 lg:grid-cols-5">
                                {balances.map((balance) => (
                                    <button
                                        key={getBalanceKey(balance)}
                                        className={cn(
                                            "w-full text-left rounded-lg border p-4 transition-all hover:border-primary/50 hover:bg-muted/40",
                                            selectedBalanceKeys.includes(getBalanceKey(balance)) && "border-primary bg-primary/5 ring-2 ring-primary"
                                        )}
                                        onClick={() => handleInvoiceSelect(balance)}
                                    >
                                        <div className="space-y-2">
                                            <div className="flex items-start justify-between gap-2">
                                                <div className="flex items-center gap-2 min-w-0">
                                                    <span className="font-semibold text-base truncate">{balance.serie} {balance.numar}</span>
                                                    <span className="text-[11px] text-muted-foreground bg-secondary px-2 py-0.5 rounded whitespace-nowrap">
                                                        {balance.tip_document || "Factura"}
                                                    </span>
                                                </div>
                                                {selectedBalanceKeys.includes(getBalanceKey(balance)) && (
                                                    <span className="text-[11px] font-medium text-primary">
                                                        Selectată #{selectedBalanceKeys.indexOf(getBalanceKey(balance)) + 1}
                                                    </span>
                                                )}
                                            </div>

                                            <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground">
                                                <div>
                                                    Data: {balance.data ? format(new Date(balance.data), "dd.MM.yyyy") : "-"}
                                                </div>
                                                <div>
                                                    Scadență: {balance.termen ? format(new Date(balance.termen), "dd.MM.yyyy") : "-"}
                                                </div>
                                                <div className="col-span-2 truncate">
                                                    Cod document: {balance.cod_document || "-"}
                                                </div>
                                                <div className="col-span-2 truncate">
                                                    Sediu: {balance.sediu || "-"}
                                                </div>
                                            </div>

                                            <div className="flex items-end justify-between gap-2 pt-1 border-t">
                                                <div>
                                                    <div className="text-[11px] text-muted-foreground">Total factură</div>
                                                    <div className="text-sm font-medium">
                                                        {new Intl.NumberFormat('ro-RO', { style: 'currency', currency: balance.moneda || 'RON' }).format(balance.valoare || 0)}
                                                    </div>
                                                </div>
                                                <div className="text-right">
                                                    <div className="text-[11px] text-muted-foreground">Rest de plată</div>
                                                    <div className="font-bold text-base text-primary">
                                                        {new Intl.NumberFormat('ro-RO', { style: 'currency', currency: balance.moneda || 'RON' }).format(balance.rest || 0)}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </button>
                                ))}
                            </div>
                        )}
                    </CardContent>
                </Card>
            )}

            {step === "details" && selectedBalances.length > 0 && (
                <Card>
                    <CardHeader>
                        <CardTitle>Detalii Plată</CardTitle>
                        <CardDescription>
                            Configurează suma pentru fiecare factură selectată
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                            <div className="rounded-lg border bg-muted/30 p-4">
                                <Label className="text-xs text-muted-foreground">Total sold selectat</Label>
                                <div className="text-3xl font-bold mt-1 text-primary">
                                    {new Intl.NumberFormat('ro-RO', { style: 'currency', currency: 'RON' }).format(
                                        selectedBalances.reduce((sum, b) => sum + (b.rest || 0), 0)
                                    )}
                                </div>
                            </div>
                            <div className="rounded-lg border bg-muted/20 p-4">
                                <Label className="text-xs text-muted-foreground">Total încasat acum</Label>
                                <div className="text-2xl font-semibold mt-1 text-foreground/80">
                                    {new Intl.NumberFormat('ro-RO', { style: 'currency', currency: 'RON' }).format(
                                        selectedBalances.reduce((sum, balance) => {
                                            const key = getBalanceKey(balance);
                                            return sum + parseAllocated(key);
                                        }, 0)
                                    )}
                                </div>
                            </div>
                        </div>

                        <div className="space-y-3">
                            {selectedBalances.map((balance, index) => {
                                const key = getBalanceKey(balance);
                                const rest = balance.rest || 0;
                                const value = allocatedAmounts[key] || "";
                                const isValid = isAllocationValid(balance);

                                return (
                                    <div key={key} className={cn("rounded-lg border p-3", !isValid && "border-red-500")}> 
                                        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                                            <div>
                                                <div className="text-sm font-semibold">
                                                    #{index + 1} • {balance.serie} {balance.numar}
                                                </div>
                                                <div className="text-xs text-muted-foreground">
                                                    Sold disponibil: {new Intl.NumberFormat('ro-RO', { style: 'currency', currency: balance.moneda || 'RON' }).format(rest)}
                                                </div>
                                            </div>
                                            <div className="w-full sm:w-52">
                                                <Label htmlFor={`alloc-${index}`} className="text-xs">Suma încasată</Label>
                                                <Input
                                                    id={`alloc-${index}`}
                                                    type="number"
                                                    step="0.01"
                                                    value={value}
                                                    onChange={(e) => {
                                                        const next = e.target.value;
                                                        setAllocatedAmounts((prev) => ({ ...prev, [key]: next }));
                                                    }}
                                                    className="h-10 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                                                />
                                                {!isValid && (
                                                    <p className="mt-1 text-xs text-red-600">{"Valoarea trebuie să fie > 0 și ≤ sold."}</p>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    </CardContent>
                </Card>
            )}
            </div>
        </div>
    );
}
