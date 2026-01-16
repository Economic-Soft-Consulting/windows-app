"use client";

import { useState, useEffect } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { usePartners } from "@/hooks/usePartners";
import { useProducts } from "@/hooks/useProducts";
import { useSyncStatus } from "@/hooks/useSyncStatus";
import { Search, Building2, Package, MapPin, Loader2, Trash2 } from "lucide-react";
import { clearDatabase } from "@/lib/tauri/commands";
import { toast } from "sonner";

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat("ro-RO", {
    style: "decimal",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount) + " RON";
}

export default function DataPage() {
  const [partnerSearch, setPartnerSearch] = useState("");
  const [productSearch, setProductSearch] = useState("");
  const [isClearing, setIsClearing] = useState(false);

  const { partners, isLoading: partnersLoading, search: searchPartners, refresh: refreshPartners } = usePartners();
  const { products, isLoading: productsLoading, search: searchProducts, refresh: refreshProducts } = useProducts();
  const { isSyncing } = useSyncStatus();

  // Listen for sync completion event from anywhere in the app
  useEffect(() => {
    const handleSyncCompleted = () => {
      console.log("Sync completed event received, refreshing data...");
      setTimeout(() => {
        refreshPartners();
        refreshProducts();
      }, 500);
    };

    window.addEventListener('sync-completed', handleSyncCompleted);
    return () => window.removeEventListener('sync-completed', handleSyncCompleted);
  }, [refreshPartners, refreshProducts]);

  const handlePartnerSearch = (query: string) => {
    setPartnerSearch(query);
    searchPartners(query);
  };

  const handleProductSearch = (query: string) => {
    setProductSearch(query);
    searchProducts(query);
  };

  const handleClearDatabase = async () => {
    if (!confirm("Sigur vrei să ștergi toate datele (parteneri, locații, produse)? Această acțiune nu poate fi anulată!")) {
      return;
    }

    setIsClearing(true);
    try {
      await clearDatabase();
      toast.success("Baza de date a fost curățată cu succes");
      refreshPartners();
      refreshProducts();
    } catch (error) {
      console.error("Failed to clear database:", error);
      toast.error("Eroare la curățarea bazei de date");
    } finally {
      setIsClearing(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Date</h1>
          <p className="text-muted-foreground">
            Vizualizează partenerii și produsele disponibile
          </p>
        </div>
        <Button
          variant="destructive"
          size="sm"
          onClick={handleClearDatabase}
          disabled={isClearing}
        >
          {isClearing ? (
            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
          ) : (
            <Trash2 className="h-4 w-4 mr-2" />
          )}
          Curăță baza de date
        </Button>
      </div>

      <Tabs defaultValue="partners">
        <TabsList className="h-14">
          <TabsTrigger value="partners" className="h-11 px-4 gap-2">
            <Building2 className="h-4 w-4" />
            Parteneri
            <span className="bg-muted text-muted-foreground px-2 py-0.5 rounded-full text-xs">
              {partners.length}
            </span>
          </TabsTrigger>
          <TabsTrigger value="products" className="h-11 px-4 gap-2">
            <Package className="h-4 w-4" />
            Produse
            <span className="bg-muted text-muted-foreground px-2 py-0.5 rounded-full text-xs">
              {products.length}
            </span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="partners" className="mt-6">
          <div className="space-y-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Caută partener..."
                value={partnerSearch}
                onChange={(e) => handlePartnerSearch(e.target.value)}
                className="pl-10 h-12"
              />
            </div>

            {partnersLoading ? (
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
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
              <div className="grid gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
                {partners.map((partner) => (
                  <Card key={partner.id} className="text-sm">
                    <CardHeader className="pb-2 pt-3 px-3">
                      <CardTitle className="text-base truncate">{partner.name}</CardTitle>
                      <CardDescription className="text-xs">
                        {partner.locations.length} locații
                      </CardDescription>
                    </CardHeader>
                    <CardContent className="px-3 pb-3">
                      <ScrollArea className="h-20">
                        <div className="space-y-1.5">
                          {partner.locations.map((location) => (
                            <div
                              key={location.id}
                              className="flex items-start gap-1.5 text-xs"
                            >
                              <MapPin className="h-3 w-3 text-muted-foreground flex-shrink-0 mt-0.5" />
                              <div>
                                <p className="font-medium leading-tight">{location.name}</p>
                                {location.address && (
                                  <p className="text-muted-foreground text-[10px] leading-tight">
                                    {location.address}
                                  </p>
                                )}
                              </div>
                            </div>
                          ))}
                        </div>
                      </ScrollArea>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </div>
        </TabsContent>

        <TabsContent value="products" className="mt-6">
          <div className="space-y-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Caută produs..."
                value={productSearch}
                onChange={(e) => handleProductSearch(e.target.value)}
                className="pl-10 h-12"
              />
            </div>

            {productsLoading ? (
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : products.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 text-center">
                <Package className="h-16 w-16 text-muted-foreground/50 mb-4" />
                <h3 className="text-lg font-medium">Nu există produse</h3>
                <p className="text-muted-foreground mt-1">
                  Sincronizează datele pentru a vedea produsele
                </p>
              </div>
            ) : (
              <div className="grid gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
                {products.map((product) => (
                  <Card key={product.id} className="text-sm">
                    <CardHeader className="pb-2 pt-3 px-3">
                      <div className="flex items-start justify-between gap-2">
                        <CardTitle className="text-sm leading-tight">{product.name}</CardTitle>
                        {product.class && (
                          <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                            {product.class}
                          </Badge>
                        )}
                      </div>
                    </CardHeader>
                    <CardContent className="px-3 pb-3">
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-muted-foreground">
                          UM: {product.unit_of_measure}
                        </span>
                        <span className="text-base font-bold">
                          {formatCurrency(product.price)}
                        </span>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
