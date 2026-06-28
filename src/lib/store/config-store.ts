import { Store } from "@tanstack/react-store";
import { CONFIG_PRESETS, clientConfigFromRecord, inferBaseUrl } from "@/lib/config/config-presets";
import {
  appearanceEditFromPayload,
  appearanceEditToPayload,
  clientConfigToRecord,
} from "@/lib/config/config-toml";
import type { AppConfigState, AppearanceEditEntry, LoadedAppConfig } from "@/lib/config/config.types";
import {
  DEFAULT_APPEARANCE_EDITS,
  createAppearanceEdit,
  isLegacyAppearanceEdit,
  isLegacySeedAppearanceEdits,
  migrateLegacyAppearanceEdit,
} from "@/lib/config/config.types";
import type { ClientConfig } from "@/lib/models.types";
import { DEFAULT_LOCAL_CONFIG } from "@/lib/models.types";

const STORAGE_KEY = "tibia-client-editor.config.v3";

const normalizeAppearanceEdits = (edits: unknown): AppearanceEditEntry[] => {
  if (!Array.isArray(edits)) return DEFAULT_APPEARANCE_EDITS;
  const normalized = edits.map((edit) =>
    isLegacyAppearanceEdit(edit) ? migrateLegacyAppearanceEdit(edit) : (edit as AppearanceEditEntry),
  );
  return isLegacySeedAppearanceEdits(normalized) ? DEFAULT_APPEARANCE_EDITS : normalized;
};

const createInitialState = (): AppConfigState => ({
  loaded: false,
  source: "ui",
  filePath: "",
  appearanceSource: "ui",
  appearanceFilePath: "",
  baseUrl: CONFIG_PRESETS.local.baseUrl,
  urls: DEFAULT_LOCAL_CONFIG,
  appearanceEdits: DEFAULT_APPEARANCE_EDITS,
});

export const configStore = new Store<AppConfigState>(createInitialState());

const persistState = (state: AppConfigState): void => {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({
      source: state.source,
      filePath: state.filePath,
      appearanceSource: state.appearanceSource,
      appearanceFilePath: state.appearanceFilePath,
      baseUrl: state.baseUrl,
      urls: state.urls,
      appearanceEdits: state.appearanceEdits,
    }),
  );
};

export const hydrateConfigStore = (): void => {
  if (typeof window === "undefined") return;

  const raw =
    window.localStorage.getItem(STORAGE_KEY) ??
    window.localStorage.getItem("tibia-client-editor.config.v2") ??
    window.localStorage.getItem("tibia-client-editor.config.v1");
  if (!raw) {
    configStore.setState((state) => ({ ...state, loaded: true }));
    return;
  }

  try {
    const parsed = JSON.parse(raw) as Partial<AppConfigState>;
    configStore.setState((state) => ({
      ...state,
      loaded: true,
      source: parsed.source ?? "ui",
      filePath: parsed.filePath ?? "",
      appearanceSource: parsed.appearanceSource ?? parsed.source ?? "ui",
      appearanceFilePath: parsed.appearanceFilePath ?? parsed.filePath ?? "",
      baseUrl: parsed.baseUrl ?? inferBaseUrl(parsed.urls ?? DEFAULT_LOCAL_CONFIG),
      urls: parsed.urls ?? DEFAULT_LOCAL_CONFIG,
      appearanceEdits: normalizeAppearanceEdits(parsed.appearanceEdits),
    }));
  } catch {
    configStore.setState((state) => ({ ...state, loaded: true }));
  }
};

const updateState = (updater: (state: AppConfigState) => AppConfigState): void => {
  configStore.setState((state) => {
    const next = updater(state);
    persistState(next);
    return next;
  });
};

export const setConfigSource = (source: AppConfigState["source"]): void => {
  updateState((state) => ({ ...state, source }));
};

export const setAppearanceSource = (appearanceSource: AppConfigState["appearanceSource"]): void => {
  updateState((state) => ({ ...state, appearanceSource }));
};

export const setConfigFilePath = (filePath: string): void => {
  updateState((state) => ({ ...state, filePath }));
};

export const setAppearanceFilePath = (appearanceFilePath: string): void => {
  updateState((state) => ({ ...state, appearanceFilePath }));
};

export const setBaseUrl = (baseUrl: string): void => {
  updateState((state) => ({ ...state, baseUrl }));
};

export const setUrls = (urls: ClientConfig): void => {
  updateState((state) => ({
    ...state,
    urls,
    baseUrl: inferBaseUrl(urls),
  }));
};

export const setUrlField = (key: keyof ClientConfig, value: string): void => {
  updateState((state) => ({
    ...state,
    urls: { ...state.urls, [key]: value },
  }));
};

export const applyBaseUrlToConfig = (baseUrl: string, urls: ClientConfig): void => {
  updateState((state) => ({
    ...state,
    baseUrl,
    urls,
  }));
};

export const setAppearanceEdits = (appearanceEdits: AppConfigState["appearanceEdits"]): void => {
  updateState((state) => ({ ...state, appearanceEdits }));
};

export const addAppearanceEdit = (entry: AppearanceEditEntry = createAppearanceEdit()): void => {
  updateState((state) => ({
    ...state,
    appearanceEdits: [...state.appearanceEdits, entry],
  }));
};

export const duplicateAppearanceEdit = (index: number): void => {
  updateState((state) => {
    const source = state.appearanceEdits[index];
    if (!source) return state;
    return {
      ...state,
      appearanceEdits: [
        ...state.appearanceEdits.slice(0, index + 1),
        {
          ...source,
          id: "",
          label: source.label ? `${source.label} copy` : "",
        },
        ...state.appearanceEdits.slice(index + 1),
      ],
    };
  });
};

export const updateAppearanceEdit = (
  index: number,
  patch: Partial<AppearanceEditEntry>,
): void => {
  updateState((state) => ({
    ...state,
    appearanceEdits: state.appearanceEdits.map((edit, editIndex) =>
      editIndex === index ? { ...edit, ...patch } : edit,
    ),
  }));
};

export const removeAppearanceEdit = (index: number): void => {
  updateState((state) => ({
    ...state,
    appearanceEdits: state.appearanceEdits.filter((_, editIndex) => editIndex !== index),
  }));
};

export const importLoadedConfig = (loaded: LoadedAppConfig, filePath?: string): void => {
  updateState((state) => ({
    ...state,
    source: filePath ? "file" : "ui",
    filePath: filePath ?? state.filePath,
    urls: clientConfigFromRecord(loaded.urls),
    baseUrl: inferBaseUrl(clientConfigFromRecord(loaded.urls)),
    appearanceEdits:
      loaded.edits.length > 0
        ? loaded.edits.map(appearanceEditFromPayload)
        : state.appearanceEdits,
  }));
};

export const importAppearanceEdits = (
  edits: LoadedAppConfig["edits"],
  filePath?: string,
): void => {
  updateState((state) => ({
    ...state,
    appearanceSource: filePath ? "file" : "ui",
    appearanceFilePath: filePath ?? state.appearanceFilePath,
    appearanceEdits: edits.map(appearanceEditFromPayload),
  }));
};

export const getActiveUrlRecord = (state: AppConfigState): Record<string, string> =>
  clientConfigToRecord(state.urls);

export const getActiveAppearancePayload = (state: AppConfigState) =>
  state.appearanceEdits
    .filter((edit) => edit.id.trim())
    .map((edit) => appearanceEditToPayload(edit));
