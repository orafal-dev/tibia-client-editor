import type {
  AppearanceBooleanFlagKey,
  AppearanceObjectFlagKey,
} from "@/lib/config/appearance-flags.types";
import type { ClientConfig } from "@/lib/models.types";

export type ConfigSource = "ui" | "file";

export type AppearanceEditEntry = {
  id: string;
  label: string;
  booleanFlags: Partial<Record<AppearanceBooleanFlagKey, boolean>>;
  objectFlags: Partial<Record<AppearanceObjectFlagKey, true>>;
};

/** @deprecated Legacy shape kept for localStorage migration */
export type LegacyAppearanceEditEntry = {
  id: string;
  usable?: boolean;
  multiuse?: boolean;
  wrap?: boolean;
  unmove?: boolean;
  cyclopediaitem?: boolean;
};

export type AppConfigState = {
  loaded: boolean;
  source: ConfigSource;
  filePath: string;
  appearanceSource: ConfigSource;
  appearanceFilePath: string;
  baseUrl: string;
  urls: ClientConfig;
  appearanceEdits: AppearanceEditEntry[];
};

export type LoadedAppConfig = {
  urls: Record<string, string>;
  edits: Array<{
    id: string;
    fields: Record<string, unknown>;
  }>;
};

export type UrlConfigGroup = {
  id: string;
  title: string;
  description: string;
  keys: (keyof ClientConfig)[];
};

export const URL_CONFIG_GROUPS: UrlConfigGroup[] = [
  {
    id: "api",
    title: "API services",
    description: "Login, client gateway, and telemetry endpoints.",
    keys: ["loginWebService", "clientWebService", "crashReportUrl", "fpsHistoryRecipient"],
  },
  {
    id: "pages",
    title: "Web pages",
    description: "Public site URLs embedded in the client.",
    keys: [
      "tibiaPageUrl",
      "tibiaStoreGetCoinsUrl",
      "getPremiumUrl",
      "manualUrl",
      "faqUrl",
      "premiumFeaturesUrl",
      "cipSoftUrl",
    ],
  },
  {
    id: "account",
    title: "Account links",
    description: "Registration and account recovery URLs.",
    keys: ["createAccountUrl", "accessAccountUrl", "lostAccountUrl"],
  },
];

export const createAppearanceEdit = (): AppearanceEditEntry => ({
  id: "",
  label: "",
  booleanFlags: {},
  objectFlags: {},
});

export const DEFAULT_APPEARANCE_EDITS: AppearanceEditEntry[] = [];

/** Previous built-in defaults — used to reset stale localStorage on migrate. */
export const LEGACY_SEED_APPEARANCE_EDIT_IDS = ["24964", "35496", "35501"] as const;

export const isLegacySeedAppearanceEdits = (edits: AppearanceEditEntry[]): boolean => {
  if (edits.length !== LEGACY_SEED_APPEARANCE_EDIT_IDS.length) return false;
  const ids = new Set(edits.map((edit) => edit.id.trim()));
  return LEGACY_SEED_APPEARANCE_EDIT_IDS.every((id) => ids.has(id));
};

export const migrateLegacyAppearanceEdit = (
  legacy: LegacyAppearanceEditEntry,
): AppearanceEditEntry => ({
  id: legacy.id,
  label: "",
  booleanFlags: {
    ...(legacy.usable ? { usable: true } : {}),
    ...(legacy.multiuse ? { multiuse: true } : {}),
    ...(legacy.wrap ? { wrap: true } : {}),
    ...(legacy.unmove ? { unmove: false } : {}),
  },
  objectFlags: legacy.cyclopediaitem ? { cyclopediaitem: true } : {},
});

export const isLegacyAppearanceEdit = (
  value: unknown,
): value is LegacyAppearanceEditEntry =>
  typeof value === "object" &&
  value !== null &&
  "id" in value &&
  ("usable" in value || "multiuse" in value || "wrap" in value);
