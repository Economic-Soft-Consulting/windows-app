"use client";

import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { usePartners } from "@/hooks/usePartners";
import { useProducts } from "@/hooks/useProducts";
import { Search, Building2, Package, MapPin, Loader2 } from "lucide-react";

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

  const { partners, isLoading: partnersLoading, search: searchPartners } = usePartners();
  const { products, isLoading: productsLoading, search: searchProducts } = useProducts();

  const handlePartnerSearch = (query: string) => {
    setPartnerSearch(query);
    searchPartners(query);
  };

  const handleProductSearch = (query: string) => {
    setProductSearch(query);
    searchProducts(query);
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Date</h1>
        <p className="text-muted-foreground">
          Vizualizează partenerii și produsele disponibile
        </p>
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
              <div className="grid gap-4 sm:grid-cols-2 md:grid-cols-2 lg:grid-cols-3">
                {partners.map((partner) => (
                  <Card key={partner.id}>
                    <CardHeader className="pb-3">
                      <CardTitle className="text-lg truncate">{partner.name}</CardTitle>
                      <CardDescription>
                        {partner.locations.length} locații
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <ScrollArea className="h-24">
                        <div className="space-y-2">
                          {partner.locations.map((location) => (
                            <div
                              key={location.id}
                              className="flex items-start gap-2 text-sm"
                            >
                              <MapPin className="h-4 w-4 text-muted-foreground flex-shrink-0 mt-0.5" />
                              <div>
                                <p className="font-medium">{location.name}</p>
                                {location.address && (
                                  <p className="text-muted-foreground text-xs">
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
              <div className="grid gap-4 sm:grid-cols-2 md:grid-cols-2 lg:grid-cols-3">
                {products.map((product) => (
                  <Card key={product.id}>
                    <CardHeader className="pb-3">
                      <div className="flex items-start justify-between gap-2">
                        <CardTitle className="text-base">{product.name}</CardTitle>
                        {product.class && (
                          <Badge variant="secondary" className="text-xs">
                            {product.class}
                          </Badge>
                        )}
                      </div>
                    </CardHeader>
                    <CardContent>
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-muted-foreground">
                          UM: {product.unit_of_measure}
                        </span>
                        <span className="text-lg font-bold">
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
