"use client";

import { useCallback, useState } from "react";
import {
  DownloadIcon,
  FileTextIcon,
  RefreshCwIcon,
  Settings2Icon,
  UploadIcon,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Switch } from "@/components/ui/switch";
import { ConfigUrlFields } from "@/components/app/config-url-fields";
import { FilePickerField } from "@/components/app/file-picker-field";
import { PanelPage } from "@/components/app/panel-page";
import { buildConfigFromBaseUrl, CONFIG_PRESETS } from "@/lib/config/config-presets";
import {
  applyBaseUrlToConfig,
  importLoadedConfig,
  setBaseUrl,
  setConfigFilePath,
  setConfigSource,
  setUrlField,
} from "@/lib/store/config-store";
import { exportConfigToml, loadConfigFile, pickFile, saveFile } from "@/lib/tauri/commands";
import { useConfigStore } from "@/hooks/use-config-store";

export const ConfigWorkspace = () => {
  const config = useConfigStore();
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleApplyBaseUrl = useCallback(() => {
    applyBaseUrlToConfig(config.baseUrl, buildConfigFromBaseUrl(config.baseUrl));
    setMessage("Generated all service URLs from the base URL.");
    setError(null);
  }, [config.baseUrl]);

  const handlePreset = useCallback((preset: keyof typeof CONFIG_PRESETS) => {
    const selected = CONFIG_PRESETS[preset];
    applyBaseUrlToConfig(selected.baseUrl, selected.urls);
    setMessage(`Applied the ${selected.label} preset.`);
    setError(null);
  }, []);

  const handleLoadFile = useCallback(async () => {
    if (!config.filePath) {
      setError("Choose a config.toml file first.");
      return;
    }

    setLoading(true);
    setError(null);
    setMessage(null);

    try {
      const loaded = await loadConfigFile(config.filePath);
      importLoadedConfig(loaded, config.filePath);
      setMessage(`Loaded configuration from ${config.filePath}`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [config.filePath]);

  const handleImportFile = useCallback(async () => {
    const selected = await pickFile("Import config.toml", [{ name: "TOML", extensions: ["toml"] }]);
    if (typeof selected !== "string") return;

    setLoading(true);
    setError(null);

    try {
      const loaded = await loadConfigFile(selected);
      importLoadedConfig(loaded, selected);
      setConfigFilePath(selected);
      setConfigSource("file");
      setMessage(`Imported ${selected} into the editor.`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, []);

  const handleExportFile = useCallback(async () => {
    const path = await saveFile("Export config.toml", "config.toml");
    if (!path) return;

    setLoading(true);
    setError(null);

    try {
      await exportConfigToml(config, path);
      setMessage(`Exported current configuration to ${path}`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [config]);

  const handleBrowseConfigFile = useCallback(async () => {
    const selected = await pickFile("Select config.toml", [{ name: "TOML", extensions: ["toml"] }]);
    if (typeof selected === "string") {
      setConfigFilePath(selected);
    }
  }, []);

  return (
    <PanelPage
      footer={
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-3 text-sm">
            <Badge variant={config.source === "file" ? "info" : "success"}>
              {config.source === "file" ? "Using config file" : "Using UI config"}
            </Badge>
            <span className="text-muted-foreground">
              URL edits save automatically · export to share or use with Patch
            </span>
          </div>
          <div className="flex shrink-0 flex-wrap items-center gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => void handleImportFile()}
              disabled={loading}
            >
              <UploadIcon aria-hidden="true" />
              Import config.toml
            </Button>
            <Button
              type="button"
              size="lg"
              onClick={() => void handleExportFile()}
              disabled={loading}
              data-loading={loading ? "" : undefined}
            >
              {loading ? <Spinner aria-hidden="true" /> : <DownloadIcon aria-hidden="true" />}
              Export config.toml
            </Button>
          </div>
        </div>
      }
    >
      <div className="grid gap-6 px-6 py-6 lg:px-8">
        <Card>
          <CardHeader>
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div>
                <CardTitle className="flex items-center gap-2">
                  <Settings2Icon className="size-4" aria-hidden="true" />
                  Server configuration
                </CardTitle>
                <CardDescription>
                  Define service URLs for the Patch tab. Item appearance edits belong on the
                  Appearances tab.
                </CardDescription>
              </div>
              <Badge variant={config.source === "file" ? "info" : "success"}>
                {config.source === "file" ? "Using config file" : "Using UI config"}
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="grid gap-6">
            <div className="flex flex-wrap gap-2">
              {(Object.keys(CONFIG_PRESETS) as Array<keyof typeof CONFIG_PRESETS>).map(
                (presetKey) => (
                  <Button
                    key={presetKey}
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => handlePreset(presetKey)}
                    disabled={loading || config.source === "file"}
                  >
                    {CONFIG_PRESETS[presetKey].label}
                  </Button>
                ),
              )}
            </div>

            <div className="rounded-xl border p-4">
              <Field>
                <FieldLabel htmlFor="config-base-url">Server base URL</FieldLabel>
                <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
                  <Input
                    id="config-base-url"
                    value={config.baseUrl}
                    onChange={(event) => setBaseUrl(event.target.value)}
                    disabled={loading || config.source === "file"}
                    placeholder="https://myserver.com"
                    aria-label="Server base URL"
                    className="min-w-0 flex-1"
                  />
                  <Button
                    type="button"
                    className="shrink-0"
                    onClick={handleApplyBaseUrl}
                    disabled={loading || config.source === "file"}
                  >
                    Apply to all URLs
                  </Button>
                </div>
                <FieldDescription>
                  Quick-fill all embedded client URLs from one domain or local dev address.
                </FieldDescription>
              </Field>
            </div>

            <ConfigUrlFields
              urls={config.urls}
              onChange={setUrlField}
              disabled={loading || config.source === "file"}
            />

            <Collapsible className="rounded-xl border">
              <CollapsibleTrigger className="flex w-full items-center justify-between px-4 py-3 text-left font-medium">
                <span className="flex items-center gap-2">
                  <FileTextIcon className="size-4" aria-hidden="true" />
                  Config file fallback
                </span>
                <Badge variant="outline">{config.source === "file" ? "Enabled" : "Optional"}</Badge>
              </CollapsibleTrigger>
              <CollapsibleContent className="grid gap-4 border-t px-4 py-4">
                <div className="flex items-center gap-3">
                  <Switch
                    id="config-use-file"
                    checked={config.source === "file"}
                    onCheckedChange={(checked) => setConfigSource(checked ? "file" : "ui")}
                    aria-label="Use config.toml file instead of UI values"
                  />
                  <Label htmlFor="config-use-file">
                    Use config.toml file instead of the values above
                  </Label>
                </div>
                {config.source === "file" && (
                  <Alert variant="info">
                    <AlertTitle>File mode active</AlertTitle>
                    <AlertDescription>
                      Patch will read URLs from the selected file. Import a file to preview or edit
                      its values in the UI, then switch back to UI mode.
                    </AlertDescription>
                  </Alert>
                )}
                <FilePickerField
                  id="config-file-path"
                  label="Config file"
                  value={config.filePath}
                  onChange={setConfigFilePath}
                  filters={[{ name: "TOML", extensions: ["toml"] }]}
                  disabled={loading}
                />
                <div className="flex flex-wrap gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => void handleBrowseConfigFile()}
                    disabled={loading}
                  >
                    Browse
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => void handleLoadFile()}
                    disabled={loading}
                  >
                    <RefreshCwIcon aria-hidden="true" />
                    Load into UI
                  </Button>
                </div>
              </CollapsibleContent>
            </Collapsible>

            {message && (
              <Alert variant="success">
                <AlertTitle>Saved</AlertTitle>
                <AlertDescription>{message}</AlertDescription>
              </Alert>
            )}
            {error && (
              <Alert variant="error">
                <AlertTitle>Configuration error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>
      </div>
    </PanelPage>
  );
};
