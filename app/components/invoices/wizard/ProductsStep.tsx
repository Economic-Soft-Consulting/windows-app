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

interface ProductsStepProps {
  cartItems: CartItem[];
  onUpdateCart: (items: CartItem[]) => void;
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

export function ProductsStep({ cartItems, onUpdateCart }: ProductsStepProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const { products, isLoading, search } = useProducts();

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      search(searchQuery);
    }, 300);
    return () => clearTimeout(timeoutId);
  }, [searchQuery, search]);

  const addToCart = (product: Product) => {
    const existingItem = cartItems.find((item) => item.product.id === product.id);
    if (existingItem) {
      onUpdateCart(
        cartItems.map((item) =>
          item.product.id === product.id
            ? { ...item, quantity: item.quantity + 1 }
            : item
        )
      );
    } else {
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
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold">Adaugă produse</h2>
        <p className="text-muted-foreground mt-1">
          Selectează produsele și cantitățile dorite
        </p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        {/* Product Search */}
        <div className="space-y-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Caută produs..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10 h-12 text-base"
            />
          </div>

          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : products.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Package className="h-12 w-12 text-muted-foreground/50 mb-4" />
              <h3 className="text-lg font-medium">Nu s-au găsit produse</h3>
            </div>
          ) : (
            <ScrollArea className="h-[calc(100vh-420px)] min-h-[200px] max-h-[400px] pr-4">
              <div className="space-y-2">
                {products.map((product) => {
                  const inCart = getCartQuantity(product.id) > 0;
                  return (
                    <Card
                      key={product.id}
                      className={inCart ? "border-primary/50 bg-primary/5" : ""}
                    >
                      <CardContent className="p-3">
                        <div className="flex items-center gap-3">
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="font-medium truncate">
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
                            </div>
                          </div>
                          <Button
                            variant={inCart ? "secondary" : "default"}
                            className="h-11 w-11 p-0"
                            onClick={() => addToCart(product)}
                          >
                            <Plus className="h-5 w-5" />
                          </Button>
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
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <ShoppingCart className="h-5 w-5" />
            <h3 className="font-semibold">Coș ({cartItems.length} produse)</h3>
          </div>

          {cartItems.length === 0 ? (
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
              <ScrollArea className="h-[calc(100vh-480px)] min-h-[150px] max-h-[350px] pr-4">
                <div className="space-y-2">
                  {cartItems.map((item) => (
                    <Card key={item.product.id}>
                      <CardContent className="p-3">
                        <div className="flex items-center gap-3">
                          <div className="flex-1 min-w-0">
                            <p className="font-medium truncate">
                              {item.product.name}
                            </p>
                            <p className="text-sm text-muted-foreground">
                              {formatCurrency(item.product.price)} /{" "}
                              {item.product.unit_of_measure}
                            </p>
                          </div>
                          <div className="flex items-center gap-1.5">
                            <Button
                              variant="outline"
                              className="h-11 w-11 p-0"
                              onClick={() => updateQuantity(item.product.id, -1)}
                            >
                              <Minus className="h-4 w-4" />
                            </Button>
                            <Input
                              type="number"
                              value={item.quantity}
                              onChange={(e) =>
                                setQuantity(
                                  item.product.id,
                                  parseInt(e.target.value) || 0
                                )
                              }
                              className="w-16 h-11 text-center"
                              min={1}
                            />
                            <Button
                              variant="outline"
                              className="h-11 w-11 p-0"
                              onClick={() => updateQuantity(item.product.id, 1)}
                            >
                              <Plus className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              className="h-11 w-11 p-0 text-red-600 hover:text-red-700 hover:bg-red-50"
                              onClick={() => removeFromCart(item.product.id)}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        </div>
                        <div className="text-right mt-2 text-sm font-medium">
                          {formatCurrency(item.product.price * item.quantity)}
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </ScrollArea>

              <Separator />

              <div className="flex items-center justify-between p-3 bg-muted rounded-lg">
                <span className="font-semibold">Total:</span>
                <span className="text-xl font-bold">
                  {formatCurrency(totalAmount)}
                </span>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
