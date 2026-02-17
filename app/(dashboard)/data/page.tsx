"use client";

import { useState, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { usePartners } from "@/hooks/usePartners";
import { useProducts } from "@/hooks/useProducts";
import { Search, Building2, Package, MapPin, Loader2, Info, Phone, Mail, FileText, CreditCard, Calendar, Trash2 } from "lucide-react";
import { Separator } from "@/components/ui/separator";
import { BarcodeScanner } from "@/app/components/barcode/BarcodeScanner";
import { formatCurrency } from "@/lib/utils";
import { getClientBalances, syncClientBalances, deletePartnersAndLocations } from "@/lib/tauri/commands";
import type { ClientBalance } from "@/lib/tauri/types";
import { toast } from "sonner";
import { useAuth } from "@/app/contexts/AuthContext";

// Helper to display data fields
const DataField = ({ label, value, className = "" }: { label: string; value: string | number | null | undefined; className?: string }) => {
  if (value === null || value === undefined || value === "") return null;
  return (
    <div className={`flex flex-col space-y-0.5 ${className}`}>
      <span className="text-[10px] text-muted-foreground uppercase tracking-wider">{label}</span>
      <span className="text-xs font-medium leading-none break-all">{value}</span>
    </div>
  );
};

export default function DataPage() {
  const { isAdmin } = useAuth();
  const [activeTab, setActiveTab] = useState<"partners" | "products">("partners");
  const [partnerSearch, setPartnerSearch] = useState("");
  const [productSearch, setProductSearch] = useState("");
  const [locationSearch, setLocationSearch] = useState("");
  const [selectedPartnerId, setSelectedPartnerId] = useState<string | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [partnerBalances, setPartnerBalances] = useState<ClientBalance[]>([]);
  const [loadingPartnerBalances, setLoadingPartnerBalances] = useState(false);
  const [syncingPartnerBalances, setSyncingPartnerBalances] = useState(false);
  const [deletingPartners, setDeletingPartners] = useState(false);

  const { partners, isLoading: partnersLoading, search: searchPartners, refresh: refreshPartners } = usePartners();
  const { products: partnerProducts, isLoading: partnerProductsLoading, search: searchPartnerProducts, refresh: refreshPartnerProducts } = useProducts(selectedPartnerId || undefined);
  const selectedPartner = partners.find((partner) => partner.id === selectedPartnerId) || null;

  // Get all products without partner filter for products tab
  const { products: allProducts, isLoading: allProductsLoading, search: searchAllProducts } = useProducts();

  // Listen for sync completion event
  useEffect(() => {
    const handleSyncCompleted = () => {
      console.log("Sync completed event received, refreshing data...");
      setTimeout(() => {
        refreshPartners();
        refreshPartnerProducts();
      }, 500);
    };

    window.addEventListener('sync-completed', handleSyncCompleted);
    return () => window.removeEventListener('sync-completed', handleSyncCompleted);
  }, [refreshPartners, refreshPartnerProducts]);

  useEffect(() => {
    setProductSearch("");
    setLocationSearch("");
  }, [selectedPartnerId]);

  useEffect(() => {
    if (!isDialogOpen || !selectedPartnerId) {
      setPartnerBalances([]);
      return;
    }

    loadPartnerBalances(selectedPartnerId);
  }, [isDialogOpen, selectedPartnerId]);

  const loadPartnerBalances = async (partnerId: string) => {
    setLoadingPartnerBalances(true);
    try {
      const data = await getClientBalances(partnerId);
      setPartnerBalances(data);
    } catch (error) {
      console.error("Failed to load partner balances:", error);
      toast.error("Eroare la încărcarea soldurilor partenerului");
      setPartnerBalances([]);
    } finally {
      setLoadingPartnerBalances(false);
    }
  };

  const handleSyncPartnerBalances = async () => {
    if (!selectedPartnerId) return;

    setSyncingPartnerBalances(true);
    try {
      await syncClientBalances();
      await loadPartnerBalances(selectedPartnerId);
      toast.success("Solduri actualizate");
    } catch (error) {
      console.error("Failed to sync partner balances:", error);
      toast.error("Eroare la actualizarea soldurilor");
    } finally {
      setSyncingPartnerBalances(false);
    }
  };

  const handlePartnerSearch = (query: string) => {
    setPartnerSearch(query);
    searchPartners(query);
  };

  const handleDeletePartnersAndLocations = async () => {
    setDeletingPartners(true);
    try {
      const result = await deletePartnersAndLocations();
      setSelectedPartnerId(null);
      setIsDialogOpen(false);
      await refreshPartners();
      toast.success(result);
    } catch (error) {
      console.error("Failed to delete partners/locations:", error);
      toast.error(`Eroare la ștergere: ${error}`);
    } finally {
      setDeletingPartners(false);
    }
  };

  const handleProductSearch = (query: string) => {
    setProductSearch(query);
    searchPartnerProducts(query);
  };

  // Location search (client-side filter)
  const handleLocationSearch = (query: string) => {
    setLocationSearch(query);
  };

  const filteredLocations = selectedPartner?.locations.filter((loc) => {
    const q = locationSearch.trim().toLowerCase();
    if (!q) return true;
    return (loc.name || "").toLowerCase().includes(q) || (loc.localitate || "").toLowerCase().includes(q);
  }) || [];

  const manyProducts = partnerProducts.length >= 12;
  const totalPartnerBalance = partnerBalances.reduce((sum, balance) => sum + (balance.rest || 0), 0);

  // Function to create mailto/tel links safely
  const safeHref = (type: 'tel' | 'mailto', value: string | null | undefined) => {
    return value ? `${type}:${value}` : '#';
  };

  return (
    <div className="h-full w-full flex flex-col space-y-4">
      <div className="shrink-0">
        <h1 className="text-2xl font-bold">Date</h1>
        <p className="text-muted-foreground">
          Vizualizează partenerii și produsele disponibile
        </p>
      </div>

      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as "partners" | "products")} className="flex-1 flex flex-col min-h-0">
        <TabsList className="grid w-full max-w-md grid-cols-2 h-10">
          <TabsTrigger value="partners" className="text-sm h-8">Parteneri</TabsTrigger>
          <TabsTrigger value="products" className="text-sm h-8">Articole</TabsTrigger>
        </TabsList>

        {/* Partners Tab */}
        <TabsContent value="partners" className="mt-3 flex-1 flex flex-col min-h-0 gap-2">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 shrink-0">
            <h2 className="text-base font-semibold flex items-center gap-2">
              <Building2 className="h-4 w-4" />
              Lista Parteneri
              <span className="bg-muted text-muted-foreground px-2 py-0.5 rounded-full text-xs ml-2">
                {partners.length}
              </span>
            </h2>
            {isAdmin && (
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button variant="destructive" size="sm" className="gap-2">
                    <Trash2 className="h-4 w-4" />
                    Șterge parteneri + sedii
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>Confirmi ștergerea?</AlertDialogTitle>
                    <AlertDialogDescription>
                      Vor fi șterși doar partenerii și sediile care NU au facturi asociate. Datele legate de facturi rămân protejate de constrângerile bazei de date.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel disabled={deletingPartners}>Anulează</AlertDialogCancel>
                    <AlertDialogAction onClick={handleDeletePartnersAndLocations} disabled={deletingPartners}>
                      {deletingPartners ? "Se șterge..." : "Șterge"}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            )}
          </div>
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
              <Input
                placeholder="Caută partener (nume, CUI, cod)..."
                value={partnerSearch}
                onChange={(e) => handlePartnerSearch(e.target.value)}
                className="pl-10 h-14 text-base"
              />
            </div>
            <BarcodeScanner
              onScan={(code) => {
                setPartnerSearch(code);
                searchPartners(code);
              }}
            />
          </div>

          <div className="flex-1 min-h-0 overflow-y-auto overflow-x-hidden">
            {partnersLoading ? (
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
              </div>
            ) : partners.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 text-center">
                <Building2 className="h-16 w-16 text-muted-foreground/50 mb-4" />
                <h3 className="text-lg font-medium">Nu există parteneri</h3>
                <p className="text-muted-foreground mt-1">
                  Sincronizează datele pentru a vedea partenerii
                </p>
              </div>
            ) : (
              <div className="grid gap-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
                {partners.map((partner) => (
                  <Card
                    key={partner.id}
                    className="overflow-hidden hover:border-primary/50 transition-colors shadow-sm active:scale-[0.99] transition-transform flex flex-col"
                  >
                    <CardHeader className="pb-1 pt-1.5 px-2 bg-muted/30">
                      <div className="flex justify-between items-start gap-1.5">
                        <div className="space-y-0.5">
                          <CardTitle className="text-xs font-bold leading-tight line-clamp-2">
                            {partner.name}
                          </CardTitle>
                          <div className="flex flex-wrap gap-1.5 text-[10px] text-muted-foreground">
                            {partner.cif && <span className="font-mono bg-background px-1.5 py-0.5 rounded border">CUI: {partner.cif}</span>}
                            {partner.cod_intern && <span className="font-mono bg-background px-1.5 py-0.5 rounded border">COD: {partner.cod_intern}</span>}
                          </div>
                        </div>
                        {partner.inactiv === "Da" && (
                          <Badge variant="destructive" className="shrink-0 text-[10px]">Inactiv</Badge>
                        )}
                      </div>
                    </CardHeader>
                    <CardContent className="pt-1.5 pb-1 px-2 space-y-1 flex-1">
                      <div className="grid grid-cols-2 gap-y-1 gap-x-1.5">
                        <DataField label="Județ" value={partner.locations[0]?.judet} />
                        <DataField label="Localitate" value={partner.locations[0]?.localitate} />
                        <DataField label="Linii Credit" value={partner.credit_client} />
                        <DataField label="Scadență" value={partner.scadenta_la_vanzare ? `${partner.scadenta_la_vanzare} zile` : undefined} />
                      </div>
                      {partner.locations.length > 0 && (
                        <div className="text-[10px] text-muted-foreground pt-0.5 flex items-center gap-1">
                          <MapPin className="h-2.5 w-2.5" />
                          {partner.locations.length} locații
                        </div>
                      )}
                    </CardContent>
                    <CardFooter className="pt-1 pb-1.5 px-2 bg-muted/10 border-t">
                      <Button
                        variant="outline"
                        className="w-full h-8 text-xs font-medium active:bg-accent"
                        onClick={() => {
                          setSelectedPartnerId(partner.id);
                          setIsDialogOpen(true);
                        }}
                      >
                        Vezi Detalii & Prețuri
                      </Button>
                    </CardFooter>
                  </Card>
                ))}
              </div>
            )}
          </div>
        </TabsContent>

        {/* Products Tab */}
        <TabsContent value="products" className="mt-3 flex-1 flex flex-col min-h-0 gap-2">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 shrink-0">
            <h2 className="text-base font-semibold flex items-center gap-2">
              <Package className="h-4 w-4" />
              Lista Articole
              <span className="bg-muted text-muted-foreground px-2 py-0.5 rounded-full text-xs ml-2">
                {allProducts.length}
              </span>
            </h2>
          </div>
          <div className="relative shrink-0">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Caută articol (nume, cod)..."
              value={productSearch}
              onChange={(e) => {
                setProductSearch(e.target.value);
                searchAllProducts(e.target.value);
              }}
              className="pl-9 h-9 text-sm"
            />
          </div>

          {allProductsLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
            </div>
          ) : allProducts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Package className="h-16 w-16 text-muted-foreground/50 mb-4" />
              <h3 className="text-lg font-medium">Nu există articole</h3>
              <p className="text-muted-foreground mt-1">
                Sincronizează datele pentru a vedea articolele
              </p>
            </div>
          ) : (
            <div className="flex-1 min-h-0 overflow-y-auto overflow-x-hidden">
              <div className="grid gap-3 grid-cols-5 pr-4 auto-rows-min">
                {allProducts.map((product) => (
                  <Card key={product.id} className="text-sm border-l-2 border-l-primary/60 hover:border-primary/80 transition-colors">
                    <CardHeader className="pb-0.5 pt-1 px-1.5">
                      <div className="flex items-start justify-between gap-1">
                        <CardTitle className="text-sm leading-tight line-clamp-2" title={product.name}>
                          {product.name}
                        </CardTitle>
                        {product.class && (
                          <Badge variant="secondary" className="text-[11px] px-1 py-0 shrink-0">
                            {product.class}
                          </Badge>
                        )}
                      </div>
                    </CardHeader>
                    <CardContent className="px-1.5 pb-1">
                      <div className="flex items-end justify-between">
                        <div className="text-[12px] text-muted-foreground">
                          <div>UM: <span className="font-medium text-foreground">{product.unit_of_measure}</span></div>
                          <div>TVA: {product.tva_percent ? `${product.tva_percent}%` : "-"}</div>
                        </div>
                        <div className="text-right">
                          <span className="text-[11px] text-muted-foreground block">Preț</span>
                          <span className="text-sm font-bold text-primary">
                            {formatCurrency(product.price)}
                          </span>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          )}
        </TabsContent>
      </Tabs>

      {/* Partner Detail & Products Dialog */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className={`max-w-[95vw] w-full ${partnerProducts.length >= 12 ? 'max-h-[95vh] sm:max-w-6xl' : 'max-h-[90vh] sm:max-w-4xl'} overflow-y-auto flex flex-col p-0 gap-0`}>
          <DialogHeader className="p-6 pb-2 border-b bg-muted/10 shrink-0">
            <div className="flex items-center gap-3">
              <Building2 className="h-8 w-8 text-primary/80" />
              <div>
                <DialogTitle className="text-xl leading-tight">{selectedPartner?.name ?? "Detalii Partener"}</DialogTitle>
                <DialogDescription className="text-xs sm:text-sm mt-1 flex flex-wrap gap-x-4 gap-y-1">
                  {selectedPartner?.cif && <span>CUI: <span className="font-mono font-medium">{selectedPartner.cif}</span></span>}
                  {selectedPartner?.reg_com && <span>Reg.Com: <span className="font-mono font-medium">{selectedPartner.reg_com}</span></span>}
                </DialogDescription>
              </div>
            </div>
          </DialogHeader>

          <Tabs defaultValue="info" className="flex-1 flex flex-col overflow-hidden">
            <div className="px-6 py-2 border-b bg-background shrink-0 overflow-x-auto">
              <TabsList className="w-full sm:w-auto grid grid-cols-4 h-12">
                <TabsTrigger value="info" className="gap-2 text-sm"><Info className="h-4 w-4" /> Info General</TabsTrigger>
                <TabsTrigger value="locations" className="gap-2 text-sm"><MapPin className="h-4 w-4" /> Locații ({selectedPartner?.locations.length || 0})</TabsTrigger>
                <TabsTrigger value="products" className="gap-2 text-sm"><Package className="h-4 w-4" /> Oferte & Prețuri</TabsTrigger>
                <TabsTrigger value="balances" className="gap-2 text-sm"><CreditCard className="h-4 w-4" /> Solduri ({partnerBalances.length})</TabsTrigger>
              </TabsList>
            </div>

            <div className="flex-1 overflow-y-auto bg-muted/5">
              <div className="p-4 pb-8">

                {/* Tab: Info General */}
                <TabsContent value="info" className="m-0 space-y-6 pr-4">
                  {selectedPartner && (
                    <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
                      <Card className="col-span-full md:col-span-2">
                        <CardHeader className="py-3 px-4 bg-muted/30 border-b"><CardTitle className="text-sm font-medium">Info Fiscal</CardTitle></CardHeader>
                        <CardContent className="grid grid-cols-2 gap-4 p-4">
                          <DataField label="CUI / CIF" value={selectedPartner.cif} />
                          <DataField label="Reg. Comertului" value={selectedPartner.reg_com} />
                          <DataField label="Cod Intern" value={selectedPartner.cod_intern} />
                          <DataField label="Cod Extern" value={selectedPartner.cod_extern} />
                          <DataField label="Tip Partener" value={selectedPartner.tip_partener} />
                          <DataField label="TVA la Incasare" value={selectedPartner.tva_la_incasare} />
                          <DataField label="Platitor TVA" value={selectedPartner.cif?.toUpperCase().startsWith("RO") ? "Da" : "Nu"} />
                          <DataField label="Statut" value={selectedPartner.inactiv === "Da" ? "Inactiv" : "Activ"} />
                        </CardContent>
                      </Card>

                      <Card className="col-span-full md:col-span-2">
                        <CardHeader className="py-3 px-4 bg-muted/30 border-b"><CardTitle className="text-sm font-medium">Financiar</CardTitle></CardHeader>
                        <CardContent className="grid grid-cols-2 gap-4 p-4">
                          <DataField label="Banca" value="-" /> {/* Placeholder if missing */}
                          <DataField label="Cont IBAN" value="-" />
                          <DataField label="Credit Client" value={selectedPartner.credit_client} />
                          <DataField label="Moneda" value={selectedPartner.moneda} />
                          <DataField label="Scadenta Vanzare" value={selectedPartner.scadenta_la_vanzare} />
                          <DataField label="Discount Fix" value={selectedPartner.discount_fix} />
                          <DataField label="Categorie Pret" value={selectedPartner.simbol_categorie_pret} />
                        </CardContent>
                      </Card>

                      <Card className="col-span-full">
                        <CardHeader className="py-3 px-4 bg-muted/30 border-b"><CardTitle className="text-sm font-medium">Diverse</CardTitle></CardHeader>
                        <CardContent className="grid grid-cols-2 md:grid-cols-4 gap-4 p-4">
                          <DataField className="col-span-2" label="Observatii" value={selectedPartner.observatii} />
                          <DataField label="Data Adaugarii" value={selectedPartner.data_adaugarii} />
                          <DataField label="Data Nastere" value={selectedPartner.data_nastere} />
                          <DataField label="Clasa" value={selectedPartner.clasa} />
                          <DataField label="Agent" value="-" />
                        </CardContent>
                      </Card>
                    </div>
                  )}
                </TabsContent>

                {/* Tab: Locations */}
                <TabsContent value="locations" className="m-0 pr-4">
                  <div className="relative mb-2">
                    <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3 w-3 text-muted-foreground" />
                    <Input
                      placeholder="Caută locație (nume, localitate)..."
                      value={locationSearch}
                      onChange={(e) => handleLocationSearch(e.target.value)}
                      className="pl-9 h-8 text-sm"
                    />
                  </div>
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {filteredLocations.map((loc, idx) => (
                      <Card key={loc.id} className="relative overflow-hidden group hover:border-primary/50 transition-colors">
                        <CardContent className="p-4 space-y-3 pt-5">
                          <div className="flex items-start gap-3 mb-2">
                            <div className="p-2 bg-muted rounded-full shrink-0"><MapPin className="h-4 w-4" /></div>
                            <div>
                              <h4 className="font-semibold text-sm leading-tight">{loc.name}</h4>
                              <p className="text-xs text-muted-foreground mt-0.5">{loc.cod_sediu ? `Cod Sediu: ${loc.cod_sediu}` : "Sediu Secundar"}</p>
                            </div>
                          </div>
                          <Separator />
                          <div className="space-y-2">
                            {(loc.strada || loc.numar) && (
                              <div className="text-sm"><span className="text-muted-foreground text-xs block uppercase tracking-wider mb-0.5">Adresa:</span>
                                {loc.strada} {loc.numar} {loc.bloc ? `Bl.${loc.bloc}` : ''}
                              </div>
                            )}
                            <div className="grid grid-cols-2 gap-2 text-xs">
                              <DataField label="Localitate" value={loc.localitate} />
                              <DataField label="Judet" value={loc.judet} />
                              <DataField label="Tara" value={loc.tara} />
                              <DataField label="Cod Postal" value={loc.cod_postal} />
                            </div>
                            {(loc.telefon || loc.email) && (
                              <>
                                <Separator className="my-2" />
                                <div className="space-y-1.5 text-xs">
                                  {loc.telefon && (
                                    <div className="flex items-center gap-2">
                                      <Phone className="h-3 w-3 text-muted-foreground" />
                                      <a href={`tel:${loc.telefon}`} className="hover:underline text-primary">{loc.telefon}</a>
                                    </div>
                                  )}
                                  {loc.email && (
                                    <div className="flex items-center gap-2">
                                      <Mail className="h-3 w-3 text-muted-foreground" />
                                      <a href={`mailto:${loc.email}`} className="hover:underline text-primary truncate max-w-full">{loc.email}</a>
                                    </div>
                                  )}
                                </div>
                              </>
                            )}
                          </div>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                </TabsContent>

                {/* Tab: Products & Offers */}
                <TabsContent value="products" className="m-0 space-y-1 pr-4">
                  <div className="bg-background rounded-lg border p-1.5 shadow-sm sticky top-0 z-10">
                    <h3 className="text-xs font-semibold mb-1.5 flex items-center gap-1.5">
                      <Search className="h-3 w-3" /> Verifică preț și ofertă pentru acest client
                    </h3>
                  </div>

                  {partnerProductsLoading ? (
                    <div className="flex items-center justify-center py-12">
                      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                  ) : partnerProducts.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-12 text-center bg-background rounded-lg border border-dashed">
                      <Package className="h-12 w-12 text-muted-foreground/30 mb-3" />
                      <h3 className="text-base font-medium">Niciun rezultat</h3>
                      <p className="text-sm text-muted-foreground py-1">
                        Nu s-au găsit produse în ofertă sau stoc pentru "{productSearch}"
                      </p>
                    </div>
                  ) : (
                    <div className={partnerProducts.length >= 12 ? "h-[60vh] overflow-auto pr-2" : ""}>
                      <div className="grid gap-1 grid-cols-4">
                        {partnerProducts.map((product) => (
                          <Card key={product.id} className="text-sm border-l-2 border-l-primary/60">
                            <CardHeader className="pb-0.5 pt-1 px-1.5">
                              <div className="flex items-start justify-between gap-1 ">
                                <CardTitle className="text-xs leading-tight line-clamp-2">{product.name}</CardTitle>
                                {product.class && (
                                  <Badge variant="secondary" className="text-[9px] px-1 py-0 shrink-0">
                                    {product.class}
                                  </Badge>
                                )}
                              </div>
                            </CardHeader>
                            <CardContent className="px-1.5 pb-1">
                              <div className="flex items-end justify-between">
                                <div className="text-[10px] text-muted-foreground">
                                  <div>UM: <span className="font-medium text-foreground">{product.unit_of_measure}</span></div>
                                  <div>TVA: {product.tva_percent ? `${product.tva_percent}%` : "-"}</div>
                                </div>
                                <div className="text-right">
                                  <span className="text-[9px] text-muted-foreground block">Preț</span>
                                  <span className="text-sm font-bold text-primary">
                                    {formatCurrency(product.price)}
                                  </span>
                                </div>
                              </div>
                            </CardContent>
                          </Card>
                        ))}
                      </div>
                    </div>
                  )}
                </TabsContent>

                {/* Tab: Partner balances */}
                <TabsContent value="balances" className="m-0 space-y-3 pr-4">
                  <div className="flex items-center justify-between bg-background rounded-lg border p-2">
                    <div>
                      <h3 className="text-sm font-semibold">Solduri partener</h3>
                      <p className="text-xs text-muted-foreground">Sunt afișate doar soldurile pentru partenerul selectat.</p>
                    </div>
                    <div className="text-right">
                      <div className="text-xs text-muted-foreground">Total sold</div>
                      <div className={`text-sm font-bold ${totalPartnerBalance > 0 ? "text-red-600" : "text-green-600"}`}>
                        {formatCurrency(totalPartnerBalance)}
                      </div>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleSyncPartnerBalances}
                      disabled={syncingPartnerBalances || loadingPartnerBalances}
                    >
                      {syncingPartnerBalances ? <Loader2 className="h-4 w-4 animate-spin" /> : "Actualizează"}
                    </Button>
                  </div>

                  {loadingPartnerBalances ? (
                    <div className="flex items-center justify-center py-12">
                      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                  ) : partnerBalances.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-12 text-center bg-background rounded-lg border border-dashed">
                      <CreditCard className="h-12 w-12 text-muted-foreground/30 mb-3" />
                      <h3 className="text-base font-medium">Nu există solduri pentru acest partener</h3>
                      <p className="text-sm text-muted-foreground py-1">
                        Apasă „Actualizează” pentru a sincroniza soldurile.
                      </p>
                    </div>
                  ) : (
                    <div className="grid gap-2">
                      {partnerBalances.map((balance) => (
                        <Card key={`${balance.id}-${balance.cod_document}-${balance.serie}-${balance.numar}`} className="text-sm">
                          <CardContent className="p-3">
                            <div className="flex items-start justify-between gap-3">
                              <div className="space-y-1">
                                <div className="flex items-center gap-2">
                                  <div className="font-medium">
                                    {balance.tip_document || "Document"} {balance.serie || ""} {balance.numar || ""}
                                  </div>
                                  <Badge variant={(balance.rest || 0) <= 0 ? "secondary" : "destructive"} className="text-[10px]">
                                    {(balance.rest || 0) <= 0 ? "Paid" : "Unpaid"}
                                  </Badge>
                                </div>
                                <div className="text-xs text-muted-foreground flex flex-wrap gap-x-3 gap-y-1">
                                  {balance.cod_document && <span>Cod: {balance.cod_document}</span>}
                                  {balance.data && <span>Data: {balance.data}</span>}
                                  {balance.termen && <span>Scadență: {balance.termen}</span>}
                                </div>
                              </div>
                              <div className="text-right space-y-0.5">
                                <div className="text-xs text-muted-foreground">Rest de plată</div>
                                <div className="text-sm font-bold text-primary">
                                  {formatCurrency(balance.rest || 0)}
                                </div>
                                {balance.valoare !== undefined && balance.valoare !== null && (
                                  <div className="text-xs text-muted-foreground">
                                    Valoare: {formatCurrency(balance.valoare)}
                                  </div>
                                )}
                              </div>
                            </div>
                          </CardContent>
                        </Card>
                      ))}
                    </div>
                  )}
                </TabsContent>
              </div>
            </div>
          </Tabs>
        </DialogContent>
      </Dialog>
    </div >
  );
}
