"use client";

import { useState, useEffect, useCallback } from "react";
import { getPartners, searchPartners } from "@/lib/tauri/commands";
import type { PartnerWithLocations } from "@/lib/tauri/types";

export function usePartners() {
  const [partners, setPartners] = useState<PartnerWithLocations[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await getPartners();
      setPartners(data);
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
      const data = await searchPartners(query);
      setPartners(data);
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

  return { partners, isLoading, error, refresh, search };
}
