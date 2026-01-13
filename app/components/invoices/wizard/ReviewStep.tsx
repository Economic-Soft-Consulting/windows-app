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
  location: Location;
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
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold">Revizuire factură</h2>
        <p className="text-muted-foreground mt-1">
          Verifică detaliile înainte de a trimite factura
        </p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        {/* Partner & Location Info */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              <Building2 className="h-4 w-4" />
              Partener
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div>
              <p className="font-medium">{partner.name}</p>
            </div>
            <Separator />
            <div className="flex items-start gap-2 text-sm">
              <MapPin className="h-4 w-4 text-muted-foreground flex-shrink-0 mt-0.5" />
              <div>
                <p className="font-medium">{location.name}</p>
                {location.address && (
                  <p className="text-muted-foreground">{location.address}</p>
                )}
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Summary */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              <Package className="h-4 w-4" />
              Sumar
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Produse:</span>
              <span className="font-medium">{cartItems.length}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Cantitate totală:</span>
              <span className="font-medium">
                {cartItems.reduce((sum, item) => sum + item.quantity, 0)} buc
              </span>
            </div>
            <Separator />
            <div className="flex justify-between">
              <span className="font-semibold">Total:</span>
              <span className="text-xl font-bold">{formatCurrency(totalAmount)}</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Products Table */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base flex items-center gap-2">
            <FileText className="h-4 w-4" />
            Produse
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="border rounded-lg overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="min-w-0">Produs</TableHead>
                  <TableHead className="text-right whitespace-nowrap">Cantitate</TableHead>
                  <TableHead className="text-right whitespace-nowrap hidden sm:table-cell">Preț unitar</TableHead>
                  <TableHead className="text-right whitespace-nowrap">Total</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {cartItems.map((item) => (
                  <TableRow key={item.product.id}>
                    <TableCell className="font-medium min-w-0">
                      <span className="line-clamp-2">{item.product.name}</span>
                    </TableCell>
                    <TableCell className="text-right whitespace-nowrap">
                      {item.quantity} {item.product.unit_of_measure}
                    </TableCell>
                    <TableCell className="text-right whitespace-nowrap hidden sm:table-cell">
                      {formatCurrency(item.product.price)}
                    </TableCell>
                    <TableCell className="text-right font-medium whitespace-nowrap">
                      {formatCurrency(item.product.price * item.quantity)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
              <TableFooter>
                <TableRow>
                  <TableCell colSpan={2} className="text-right font-semibold sm:hidden">
                    Total
                  </TableCell>
                  <TableCell colSpan={3} className="text-right font-semibold hidden sm:table-cell">
                    Total
                  </TableCell>
                  <TableCell className="text-right font-bold text-lg whitespace-nowrap">
                    {formatCurrency(totalAmount)}
                  </TableCell>
                </TableRow>
              </TableFooter>
            </Table>
          </div>
        </CardContent>
      </Card>

      {/* Notes */}
      <div className="space-y-2">
        <Label htmlFor="notes" className="text-base">
          Note (opțional)
        </Label>
        <Textarea
          id="notes"
          placeholder="Adaugă note sau observații pentru această factură..."
          value={notes}
          onChange={(e) => onNotesChange(e.target.value)}
          className="min-h-[100px] text-base"
        />
      </div>
    </div>
  );
}
