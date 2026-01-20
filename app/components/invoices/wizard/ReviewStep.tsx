"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  TableFooter,
} from "@/components/ui/table";
import { Building2, MapPin, Package, FileText } from "lucide-react";
import type { PartnerWithLocations, Location, CartItem } from "@/lib/tauri/types";

interface ReviewStepProps {
  partner: PartnerWithLocations;
  location?: Location;
  cartItems: CartItem[];
  notes: string;
  onNotesChange: (notes: string) => void;
}

function formatCurrency(amount: number): string {
  return (
    new Intl.NumberFormat("ro-RO", {
      style: "decimal",
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(amount) + " RON"
  );
}

export function ReviewStep({
  partner,
  location,
  cartItems,
  notes,
  onNotesChange,
}: ReviewStepProps) {
  const totalAmount = cartItems.reduce(
    (sum, item) => sum + item.product.price * item.quantity,
    0
  );

  return (
    <div className="space-y-2">
      <div>
        <h2 className="text-sm font-semibold">Revizuire factură</h2>
        <p className="text-[11px] text-muted-foreground mt-0.5">
          Verifică detaliile înainte de a trimite factura
        </p>
      </div>

      <div className="grid gap-3 md:grid-cols-2">
        {/* Partner & Location Info */}
        <Card>
          <CardHeader className="pb-2 pt-2 px-2">
            <CardTitle className="text-sm flex items-center gap-1.5">
              <Building2 className="h-3.5 w-3.5" />
              Partener
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 px-2 pb-2">
            <div>
              <p className="text-sm font-medium">{partner.name}</p>
            </div>
            <Separator />
            <div className="flex items-start gap-1.5 text-xs">
              <MapPin className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0 mt-0.5" />
              <div>
                {location ? (
                  <>
                    <p className="font-medium">{location.name}</p>
                    {location.address && (
                      <p className="text-muted-foreground">{location.address}</p>
                    )}
                  </>
                ) : (
                  <p className="text-muted-foreground">Fără sediu specific</p>
                )}
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Summary */}
        <Card>
          <CardHeader className="pb-2 pt-2 px-2">
            <CardTitle className="text-sm flex items-center gap-1.5">
              <Package className="h-3.5 w-3.5" />
              Sumar
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 px-2 pb-2">
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Produse:</span>
              <span className="font-medium">{cartItems.length}</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Cantitate totală:</span>
              <span className="font-medium">
                {cartItems.reduce((sum, item) => sum + item.quantity, 0)} buc
              </span>
            </div>
            <Separator />
            <div className="flex justify-between">
              <span className="text-sm font-semibold">Total:</span>
              <span className="text-base font-bold">{formatCurrency(totalAmount)}</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Products Table */}
      <Card>
        <CardHeader className="pb-2 pt-2 px-2">
          <CardTitle className="text-sm flex items-center gap-1.5">
            <FileText className="h-3.5 w-3.5" />
            Produse
          </CardTitle>
        </CardHeader>
        <CardContent className="px-2 pb-2">
          <div className="border rounded-lg overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="min-w-0 text-xs h-8">Produs</TableHead>
                  <TableHead className="text-right whitespace-nowrap text-xs h-8">Cantitate</TableHead>
                  <TableHead className="text-right whitespace-nowrap hidden sm:table-cell text-xs h-8">Preț unitar</TableHead>
                  <TableHead className="text-right whitespace-nowrap text-xs h-8">Total</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {cartItems.map((item) => (
                  <TableRow key={item.product.id}>
                    <TableCell className="text-xs font-medium min-w-0 py-2">
                      <span className="line-clamp-2">{item.product.name}</span>
                      {item.product.tva_percent != null && (
                        <span className="text-[10px] text-muted-foreground block mt-0.5">
                          TVA: {item.product.tva_percent}%
                        </span>
                      )}
                    </TableCell>
                    <TableCell className="text-right whitespace-nowrap text-xs py-2">
                      {item.quantity} {item.product.unit_of_measure}
                    </TableCell>
                    <TableCell className="text-right whitespace-nowrap hidden sm:table-cell text-xs py-2">
                      {formatCurrency(item.product.price)}
                    </TableCell>
                    <TableCell className="text-right font-medium whitespace-nowrap text-xs py-2">
                      {formatCurrency(item.product.price * item.quantity)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
              <TableFooter>
                <TableRow>
                  <TableCell colSpan={2} className="text-right text-xs font-semibold sm:hidden py-2">
                    Total
                  </TableCell>
                  <TableCell colSpan={3} className="text-right text-xs font-semibold hidden sm:table-cell py-2">
                    Total
                  </TableCell>
                  <TableCell className="text-right font-bold text-sm whitespace-nowrap py-2">
                    {formatCurrency(totalAmount)}
                  </TableCell>
                </TableRow>
              </TableFooter>
            </Table>
          </div>
        </CardContent>
      </Card>

      {/* Notes */}
      <div className="space-y-1.5">
        <Label htmlFor="notes" className="text-sm">
          Note (opțional)
        </Label>
        <Textarea
          id="notes"
          placeholder="Adaugă note sau observații pentru această factură..."
          value={notes}
          onChange={(e) => onNotesChange(e.target.value)}
          className="min-h-[80px] text-sm"
        />
      </div>
    </div>
  );
}
