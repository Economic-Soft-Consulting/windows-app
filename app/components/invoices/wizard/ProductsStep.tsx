"use client";

import { useState, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useProducts } from "@/hooks/useProducts";
import {
  Search,
  Package,
  Plus,
  Minus,
  Trash2,
  ShoppingCart,
  Loader2,
} from "lucide-react";
import type { Product, CartItem } from "@/lib/tauri/types";
import { formatCurrency } from "@/lib/utils";

interface ProductsStepProps {
  cartItems: CartItem[];
  onUpdateCart: (items: CartItem[]) => void;
  partnerId?: string;
}

export function ProductsStep({ cartItems, onUpdateCart, partnerId }: ProductsStepProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const { products, isLoading, search } = useProducts(partnerId);

  // Filter products to show only eggs (ouă) class
  const filteredProducts = products.filter(product => {
    const productClass = product.class?.toLowerCase() || "";
    return productClass.includes("ou") || productClass.includes("oua");
  });

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      search(searchQuery);
    }, 300);
    return () => clearTimeout(timeoutId);
  }, [searchQuery, search]);

  const addToCart = (product: Product) => {
    const existingItem = cartItems.find((item) => item.product.id === product.id);
    if (existingItem) {
      // Already in cart - do nothing
      return;
    } else {
      // Add with quantity 1 and remove from the visible product list
      onUpdateCart([...cartItems, { product, quantity: 1 }]);
    }
  };

  const updateQuantity = (productId: string, delta: number) => {
    onUpdateCart(
      cartItems
        .map((item) =>
          item.product.id === productId
            ? { ...item, quantity: Math.max(0, item.quantity + delta) }
            : item
        )
        .filter((item) => item.quantity > 0)
    );
  };

  const setQuantity = (productId: string, quantity: number) => {
    if (quantity <= 0) {
      onUpdateCart(cartItems.filter((item) => item.product.id !== productId));
    } else {
      onUpdateCart(
        cartItems.map((item) =>
          item.product.id === productId ? { ...item, quantity } : item
        )
      );
    }
  };

  const removeFromCart = (productId: string) => {
    onUpdateCart(cartItems.filter((item) => item.product.id !== productId));
  };

  const totalAmount = cartItems.reduce(
    (sum, item) => sum + item.product.price * item.quantity,
    0
  );

  const getCartQuantity = (productId: string): number => {
    const item = cartItems.find((item) => item.product.id === productId);
    return item?.quantity ?? 0;
  };

  return (
    <div className="space-y-2">
      <div className="grid gap-2.5 md:grid-cols-[1fr_1fr]">
        {/* Left Column Header */}
        <div>
          <h2 className="text-xl font-semibold">Adaugă produse</h2>
          <p className="text-[14px] text-muted-foreground mt-0.5">
            Selectează produsele și cantitățile dorite
          </p>
        </div>
        
        {/* Right Column Header */}
        <div className="flex items-center gap-1.5">
          <ShoppingCart className="h-4 w-4" />
          <h2 className="text-sm font-semibold">Coș ({cartItems.length})</h2>
        </div>
      </div>

      <div className="grid gap-2.5 md:grid-cols-[1fr_1fr]">
        {/* Product Search */}
        <div className="space-y-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
            <Input
              placeholder="Caută produs..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-8 w-118 h-9 text-sm"
            />
          </div>

          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : filteredProducts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Package className="h-12 w-12 text-muted-foreground/50 mb-4" />
              <h3 className="text-lg font-medium">Nu s-au găsit produse din clasa ouă</h3>
              <p className="text-xs text-muted-foreground mt-1">Numai produsele din clasa ouă sunt afișate</p>
            </div>
          ) : (
            <ScrollArea className="h-[calc(100vh-270px)] min-h-[280px] max-h-[550px]">
              <div className="space-y-2 pr-4">
                {filteredProducts.filter(p => !cartItems.some(ci => ci.product.id === p.id)).map((product) => {
                  return (
                    <Card
                      key={product.id}
                      className={`cursor-pointer transition-all hover:border-primary/50 min-h-[64px]`}
                      onClick={() => addToCart(product)}
                    >
                      <CardContent className="p-2.5">
                        <div className="flex items-center justify-between gap-3">
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="text-sm font-medium truncate">
                                {product.name}
                              </span>
                              {product.class && (
                                <Badge variant="secondary" className="text-xs">
                                  {product.class}
                                </Badge>
                              )}
                            </div>
                            <div className="flex items-center gap-2 mt-1">
                              <span className="text-sm text-muted-foreground">
                                {product.unit_of_measure}
                              </span>
                              <span className="text-sm font-medium">
                                {formatCurrency(product.price)}
                              </span>
                              {product.tva_percent != null && (
                                <Badge variant="outline" className="text-xs">
                                  TVA {product.tva_percent}%
                                </Badge>
                              )}
                            </div>
                          </div>
                          <Plus className="h-5 w-5 text-muted-foreground flex-shrink-0" />
                        </div>
                      </CardContent>
                    </Card>
                  );
                })}
              </div>
            </ScrollArea>
          )}
        </div>

        {/* Cart */}
        <div className="space-y-2">{cartItems.length === 0 ? (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12 text-center">
                <ShoppingCart className="h-12 w-12 text-muted-foreground/50 mb-4" />
                <p className="text-muted-foreground">
                  Adaugă produse din lista de mai sus
                </p>
              </CardContent>
            </Card>
          ) : (
            <>
              <ScrollArea className="h-[calc(100vh-270px)] min-h-[280px] max-h-[550px] pr-4">
                <div className="space-y-2">
                  {cartItems.map((item) => (
                  <Card key={item.product.id} className="min-h-[64px]">
                      <CardContent className="p-2">
                        <div className="flex items-center gap-2">
                          <div className="flex-1 min-w-0">
                            <p className="text-sm font-medium truncate">
                              {item.product.name}
                            </p>
                            <p className="text-sm text-muted-foreground">
                              {formatCurrency(item.product.price)} /{" "}
                              {item.product.unit_of_measure}
                            </p>
                            {item.product.tva_percent != null && (
                              <p className="text-xs text-muted-foreground mt-0.5">
                                TVA: {item.product.tva_percent}%
                              </p>
                            )}
                          </div>
                          <div className="flex items-center gap-1">
                            <Button
                              variant="outline"
                              className="h-9 w-9 p-0"
                              onClick={() => updateQuantity(item.product.id, -1)}
                            >
                              <Minus className="h-4 w-4" />
                            </Button>
                            <Input
                              type="number"
                              inputMode="numeric"
                              value={item.quantity}
                              onChange={(e) =>
                                setQuantity(
                                  item.product.id,
                                  parseInt(e.target.value) || 0
                                )
                              }
                              onFocus={(e) => e.target.select()}
                              className="w-24 h-9 text-center text-sm"
                              min={0}
                            />
                            <Button
                              variant="outline"
                              className="h-9 w-9 p-0"
                              onClick={() => updateQuantity(item.product.id, 1)}
                            >
                              <Plus className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              className="h-9 w-9 p-0 text-red-600 hover:text-red-700 hover:bg-red-50"
                              onClick={() => removeFromCart(item.product.id)}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        </div>
                        <div className="text-right mt-1 text-sm font-medium">
                          {formatCurrency(item.product.price * item.quantity)}
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </ScrollArea>

              <div className="mt-3">
                <Separator className="mb-2" />

                <div className="flex items-center justify-between p-2 bg-muted rounded-lg">
                  <span className="text-xs font-semibold">Total:</span>
                  <span className="text-sm font-bold">
                    {formatCurrency(totalAmount)}
                  </span>
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
