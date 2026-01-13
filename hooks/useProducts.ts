"use client";

import { useState, useEffect, useCallback } from "react";
import { getProducts, searchProducts } from "@/lib/tauri/commands";
import type { Product } from "@/lib/tauri/types";

export function useProducts() {
  const [products, setProducts] = useState<Product[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await getProducts();
      setProducts(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, []);

  const search = useCallback(async (query: string) => {
    if (!query.trim()) {
      return refresh();
    }
    setIsLoading(true);
    try {
      const data = await searchProducts(query);
      setProducts(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, [refresh]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { products, isLoading, error, refresh, search };
}
