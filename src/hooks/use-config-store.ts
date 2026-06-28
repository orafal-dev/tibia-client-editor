"use client";

import { useEffect } from "react";
import { useStore } from "@tanstack/react-store";
import { configStore, hydrateConfigStore } from "@/lib/store/config-store";

export const useConfigStore = () => useStore(configStore);

export const useHydrateConfigStore = (): boolean => {
  const loaded = useStore(configStore, (state) => state.loaded);

  useEffect(() => {
    hydrateConfigStore();
  }, []);

  return loaded;
};
