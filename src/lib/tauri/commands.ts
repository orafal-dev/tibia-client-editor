import { invoke } from "@tauri-apps/api/core";
import type { LoadedAppConfig } from "@/lib/config/config.types";
import {
  serializeConfigToml,
} from "@/lib/config/config-toml";
import type {
  AppearancesResult,
  DiagnoseOutput,
  EditOutput,
  RepackPlatform,
  RepackResult,
  Win2MacResult,
} from "@/lib/models.types";
import type { AppConfigState } from "@/lib/config/config.types";
import {
  getActiveAppearancePayload,
  getActiveUrlRecord,
} from "@/lib/store/config-store";

export const loadConfigFile = async (configPath: string): Promise<LoadedAppConfig> =>
  invoke<LoadedAppConfig>("load_config_file", {
    args: { config_path: configPath },
  });

export const exportConfigToml = async (
  state: AppConfigState,
  path: string,
): Promise<void> => {
  const { writeTextFile } = await import("@tauri-apps/plugin-fs");
  await writeTextFile(path, serializeConfigToml(state.urls, state.appearanceEdits));
};

export const editClient = async (args: {
  tibiaExe: string;
  config: AppConfigState;
  sourceTibiaExe?: string;
  strictClientCheck?: boolean;
  aggressiveClientCheck?: boolean;
}): Promise<EditOutput> =>
  invoke<EditOutput>("edit_client", {
    args: {
      tibia_exe: args.tibiaExe,
      config_path: args.config.source === "file" && args.config.filePath ? args.config.filePath : null,
      config_values: args.config.source === "ui" ? getActiveUrlRecord(args.config) : null,
      source_tibia_exe: args.sourceTibiaExe ?? null,
      strict_client_check: args.strictClientCheck ?? false,
      aggressive_client_check: args.aggressiveClientCheck ?? false,
    },
  });

export const diagnoseClient = async (args: {
  tibiaExe: string;
  compareWith?: string;
  strictClientCheck?: boolean;
}): Promise<DiagnoseOutput> =>
  invoke<DiagnoseOutput>("diagnose_client", {
    args: {
      tibia_exe: args.tibiaExe,
      compare_with: args.compareWith ?? null,
      strict_client_check: args.strictClientCheck ?? false,
    },
  });

export const repackClient = async (args: {
  src: string;
  dst: string;
  platform: RepackPlatform;
}): Promise<RepackResult> =>
  invoke<RepackResult>("repack_client", {
    args: {
      src: args.src,
      dst: args.dst,
      platform: args.platform,
    },
  });

export const win2macAssets = async (args: {
  src: string;
  dst: string;
}): Promise<Win2MacResult> =>
  invoke<Win2MacResult>("win2mac_assets", {
    args: {
      src: args.src,
      dst: args.dst,
    },
  });

export const editAppearances = async (args: {
  appearancesPath: string;
  config: AppConfigState;
  outputPath?: string;
}): Promise<AppearancesResult> =>
  invoke<AppearancesResult>("edit_appearances", {
    args: {
      appearances_path: args.appearancesPath,
      config_path:
        args.config.appearanceSource === "file" && args.config.appearanceFilePath
          ? args.config.appearanceFilePath
          : null,
      edits:
        args.config.appearanceSource === "ui" ? getActiveAppearancePayload(args.config) : null,
      output_path: args.outputPath ?? null,
    },
  });

export const pickFile = async (title: string, filters?: { name: string; extensions: string[] }[]) => {
  const { open } = await import("@tauri-apps/plugin-dialog");
  return open({ title, multiple: false, filters });
};

export const pickDirectory = async (title: string) => {
  const { open } = await import("@tauri-apps/plugin-dialog");
  return open({ title, directory: true, multiple: false });
};

export const saveFile = async (title: string, defaultPath?: string) => {
  const { save } = await import("@tauri-apps/plugin-dialog");
  return save({ title, defaultPath });
};

export const isTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
