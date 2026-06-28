import type { ClientConfig } from "@/lib/models.types";
import { DEFAULT_LOCAL_CONFIG, URL_PROPERTY_KEYS } from "@/lib/models.types";

const normalizeBaseUrl = (baseUrl: string): string => {
  const trimmed = baseUrl.trim().replace(/\/+$/, "");
  return trimmed.endsWith("/") ? trimmed : `${trimmed}/`;
};

export const buildConfigFromBaseUrl = (baseUrl: string): ClientConfig => {
  const base = normalizeBaseUrl(baseUrl);
  const root = base.endsWith("/") ? base.slice(0, -1) : base;

  return {
    loginWebService: `${root}/api/login`,
    clientWebService: `${root}/api/login`,
    tibiaPageUrl: `${base}`,
    tibiaStoreGetCoinsUrl: `${root}/shop/coins`,
    getPremiumUrl: `${root}/pages/vip-features`,
    createAccountUrl: `${root}/account/signup`,
    accessAccountUrl: `${root}/account`,
    lostAccountUrl: `${root}/account/lost`,
    manualUrl: `${root}/pages/server-info`,
    faqUrl: `${root}/pages/server-info`,
    premiumFeaturesUrl: `${root}/pages/vip-features`,
    crashReportUrl: `${root}/api/crash-report`,
    cipSoftUrl: `${base}`,
    fpsHistoryRecipient: `${root}/api/hardware-report`,
  };
};

export const CONFIG_PRESETS = {
  local: {
    label: "Local dev",
    description: "SlenderAAC / local launcher on port 7171",
    baseUrl: "http://127.0.0.1:7171",
    urls: DEFAULT_LOCAL_CONFIG,
  },
  production: {
    label: "Production",
    description: "Template for a public OT server domain",
    baseUrl: "https://example.com",
    urls: buildConfigFromBaseUrl("https://example.com"),
  },
} as const;

export const clientConfigFromRecord = (record: Record<string, string>): ClientConfig => {
  const next = { ...DEFAULT_LOCAL_CONFIG };
  for (const key of URL_PROPERTY_KEYS) {
    if (record[key]) {
      next[key] = record[key];
    }
  }
  return next;
};

export const inferBaseUrl = (urls: ClientConfig): string => {
  try {
    const parsed = new URL(urls.tibiaPageUrl);
    return `${parsed.protocol}//${parsed.host}`;
  } catch {
    return CONFIG_PRESETS.local.baseUrl;
  }
};
