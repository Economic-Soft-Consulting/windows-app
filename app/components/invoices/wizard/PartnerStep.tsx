"use client";

import { useState, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { usePartners } from "@/hooks/usePartners";
import { Search, Building2, MapPin, Check, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import type { PartnerWithLocations } from "@/lib/tauri/types";

interface PartnerStepProps {
  selectedPartner: PartnerWithLocations | null;
  onSelect: (partner: PartnerWithLocations) => void;
}

export function PartnerStep({ selectedPartner, onSelect }: PartnerStepProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const { partners, isLoading, search } = usePartners();

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      search(searchQuery);
    }, 300);
    return () => clearTimeout(timeoutId);
  }, [searchQuery, search]);

  return (
    <div className="space-y-2">
      <div>
        <h2 className="text-base font-semibold">Selectează partenerul</h2>
        <p className="text-xs text-muted-foreground mt-0.5">
          Alege partenerul pentru care dorești să creezi factura
        </p>
      </div>

      <div className="relative">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
        <Input
          placeholder="Caută partener după nume..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="pl-9 h-9 text-sm"
        />
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : partners.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-12 text-center">
          <Building2 className="h-12 w-12 text-muted-foreground/50 mb-4" />
          <h3 className="text-lg font-medium">Nu s-au găsit parteneri</h3>
          <p className="text-muted-foreground mt-1">
            Încearcă cu alt termen de căutare
          </p>
        </div>
      ) : (
        <ScrollArea className="h-[calc(100vh-280px)] min-h-[300px] max-h-[600px] pr-4">
          <div className="grid gap-2 grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
            {partners.map((partner) => {
              const isSelected = selectedPartner?.id === partner.id;
              return (
                <Card
                  key={partner.id}
                  className={cn(
                    "cursor-pointer transition-all hover:border-primary/50 min-h-[68px]",
                    isSelected && "border-primary bg-primary/5 ring-2 ring-primary"
                  )}
                  onClick={() => onSelect(partner)}
                >
                  <CardHeader className="pb-1 pt-1.5 px-2">
                    <div className="flex items-start justify-between gap-1.5">
                      <CardTitle className="text-sm leading-tight line-clamp-2">{partner.name}</CardTitle>
                      {isSelected && (
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
                          {partner.scadenta_la_vanzare || '7'}z
                        </span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
