import type { AppearanceEditEntry } from "@/lib/config/config.types";
import type {
  AppearanceBooleanFlagKey,
  AppearanceObjectFlagKey,
} from "@/lib/config/appearance-flags.types";
import type { ClientConfig } from "@/lib/models.types";
import { URL_PROPERTY_KEYS } from "@/lib/models.types";

const serializeTomlValue = (value: unknown): string => {
  if (typeof value === "string") {
    return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  if (typeof value === "number") {
    return String(value);
  }
  if (value && typeof value === "object" && Object.keys(value).length === 0) {
    return "{}";
  }
  return "null";
};

const serializeAppearanceEditBlock = (edit: AppearanceEditEntry): string => {
  const lines: string[] = ["[[edit]]"];
  if (edit.label.trim()) {
    lines.push(`# ${edit.label.trim()}`);
  }
  lines.push(`id = ${serializeTomlValue(edit.id.trim())}`);

  const booleanKeys = Object.keys(edit.booleanFlags).sort() as AppearanceBooleanFlagKey[];
  for (const key of booleanKeys) {
    const value = edit.booleanFlags[key];
    if (value !== undefined) {
      lines.push(`${key} = ${serializeTomlValue(value)}`);
    }
  }

  const objectKeys = Object.keys(edit.objectFlags).sort() as AppearanceObjectFlagKey[];
  for (const key of objectKeys) {
    lines.push(`${key} = {}`);
  }

  return lines.join("\n");
};

export const serializeConfigToml = (
  urls: ClientConfig,
  appearanceEdits: AppearanceEditEntry[],
): string => {
  const urlLines = URL_PROPERTY_KEYS.map((key) => `${key} = ${serializeTomlValue(urls[key])}`);
  const editBlocks = appearanceEdits
    .filter((edit) => edit.id.trim())
    .map(serializeAppearanceEditBlock);

  return `${[...urlLines, "", ...editBlocks].join("\n").trim()}\n`;
};

export const serializeAppearanceEditsToml = (appearanceEdits: AppearanceEditEntry[]): string => {
  const blocks = appearanceEdits
    .filter((edit) => edit.id.trim())
    .map(serializeAppearanceEditBlock);
  return blocks.length > 0 ? `${blocks.join("\n\n")}\n` : "";
};

export const appearanceEditToPayload = (edit: AppearanceEditEntry) => {
  const fields: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(edit.booleanFlags)) {
    if (value !== undefined) {
      fields[key] = value;
    }
  }

  for (const key of Object.keys(edit.objectFlags) as AppearanceObjectFlagKey[]) {
    fields[key] = {};
  }

  return { id: edit.id.trim(), fields };
};

const OBJECT_FLAG_KEYS = new Set<string>([
  "bank",
  "write",
  "write_once",
  "hook",
  "light",
  "shift",
  "height",
  "automap",
  "lenshelp",
  "clothes",
  "default_action",
  "market",
  "changedtoexpire",
  "cyclopediaitem",
  "upgradeclassification",
]);

export const appearanceEditFromPayload = (payload: {
  id: string;
  fields: Record<string, unknown>;
}): AppearanceEditEntry => {
  const booleanFlags: Partial<Record<AppearanceBooleanFlagKey, boolean>> = {};
  const objectFlags: Partial<Record<AppearanceObjectFlagKey, true>> = {};

  for (const [key, value] of Object.entries(payload.fields)) {
    if (OBJECT_FLAG_KEYS.has(key)) {
      objectFlags[key as AppearanceObjectFlagKey] = true;
      continue;
    }
    if (typeof value === "boolean") {
      booleanFlags[key as AppearanceBooleanFlagKey] = value;
    }
  }

  return {
    id: payload.id,
    label: "",
    booleanFlags,
    objectFlags,
  };
};

export const clientConfigToRecord = (urls: ClientConfig): Record<string, string> =>
  Object.fromEntries(URL_PROPERTY_KEYS.map((key) => [key, urls[key]]));
