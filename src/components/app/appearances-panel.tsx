"use client";

import { useCallback, useState } from "react";
import {
  DownloadIcon,
  RefreshCwIcon,
  SparklesIcon,
  UploadIcon,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Frame,
  FrameDescription,
  FrameHeader,
  FramePanel,
  FrameTitle,
} from "@/components/ui/frame";
import { Spinner } from "@/components/ui/spinner";
import { Tabs, TabsList, TabsTab } from "@/components/ui/tabs";
import { AppearanceItemsEditor } from "@/components/app/appearance-items-editor";
import { FilePickerField } from "@/components/app/file-picker-field";
import { LogViewer } from "@/components/app/log-viewer";
import { PanelPage } from "@/components/app/panel-page";
import { useConfigStore } from "@/hooks/use-config-store";
import { serializeAppearanceEditsToml } from "@/lib/config/config-toml";
import type { AppearancesResult } from "@/lib/models.types";
import {
  addAppearanceEdit,
  duplicateAppearanceEdit,
  importAppearanceEdits,
  removeAppearanceEdit,
  setAppearanceFilePath,
  setAppearanceSource,
  updateAppearanceEdit,
} from "@/lib/store/config-store";
import { editAppearances, loadConfigFile, pickFile, saveFile } from "@/lib/tauri/commands";

export const AppearancesPanel = () => {
  const config = useConfigStore();
  const [appearancesPath, setAppearancesPath] = useState("");
  const [outputPath, setOutputPath] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [result, setResult] = useState<AppearancesResult | null>(null);

  const handleEditorModeChange = useCallback((value: string) => {
    setAppearanceSource(value === "file" ? "file" : "ui");
  }, []);

  const handleApply = useCallback(async () => {
    if (!appearancesPath) {
      setError("Select appearances.dat to patch.");
      return;
    }

    if (config.appearanceSource === "file" && !config.appearanceFilePath) {
      setError("File mode is enabled. Choose a config.toml with [[edit]] blocks.");
      return;
    }

    if (
      config.appearanceSource === "ui" &&
      config.appearanceEdits.every((edit) => !edit.id.trim())
    ) {
      setError("Add at least one item with an ID, or switch to config file mode.");
      return;
    }

    setLoading(true);
    setError(null);
    setMessage(null);
    setResult(null);

    try {
      const output = await editAppearances({
        appearancesPath,
        config,
        outputPath: outputPath || undefined,
      });
      setResult(output);
      setMessage(`Updated ${output.edits_applied} item(s).`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [appearancesPath, config, outputPath]);

  const handleLoadEditsFile = useCallback(async () => {
    if (!config.appearanceFilePath) {
      setError("Choose a config.toml file first.");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const loaded = await loadConfigFile(config.appearanceFilePath);
      importAppearanceEdits(loaded.edits, config.appearanceFilePath);
      setSelectedIndex(0);
      setAppearanceSource("ui");
      setMessage(`Loaded ${loaded.edits.length} [[edit]] block(s) into the visual editor.`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [config.appearanceFilePath]);

  const handleImportEditsFile = useCallback(async () => {
    const selected = await pickFile("Import config.toml with [[edit]] blocks", [
      { name: "TOML", extensions: ["toml"] },
    ]);
    if (typeof selected !== "string") return;

    setLoading(true);
    setError(null);

    try {
      const loaded = await loadConfigFile(selected);
      importAppearanceEdits(loaded.edits, selected);
      setAppearanceFilePath(selected);
      setAppearanceSource("file");
      setSelectedIndex(0);
      setMessage(`Imported ${loaded.edits.length} item edit(s) from ${selected}.`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, []);

  const handleExportEdits = useCallback(async () => {
    const path = await saveFile("Export appearance edits", "appearances-edits.toml");
    if (!path) return;

    setLoading(true);
    setError(null);

    try {
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
      await writeTextFile(path, serializeAppearanceEditsToml(config.appearanceEdits));
      setMessage(`Exported item edits to ${path}`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [config.appearanceEdits]);

  const editorMode = config.appearanceSource === "file" ? "file" : "ui";

  return (
    <PanelPage
      footer={
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-3 text-sm">
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Edit source:</span>
              <Badge variant={editorMode === "file" ? "info" : "success"}>
                {editorMode === "file" ? "config.toml" : "Visual editor"}
              </Badge>
            </div>
            {editorMode === "ui" && (
              <span className="text-muted-foreground">
                {config.appearanceEdits.length} item
                {config.appearanceEdits.length === 1 ? "" : "s"} configured
              </span>
            )}
          </div>
          <Button
            type="button"
            size="lg"
            onClick={() => void handleApply()}
            disabled={loading}
            data-loading={loading ? "" : undefined}
            className="shrink-0"
          >
            {loading ? <Spinner aria-hidden="true" /> : <SparklesIcon aria-hidden="true" />}
            Apply to appearances.dat
          </Button>
        </div>
      }
    >
      <div className="flex w-full flex-col gap-8 px-6 py-6 lg:px-8">
        <Alert>
          <AlertTitle>How this works</AlertTitle>
          <AlertDescription className="space-y-2">
            <p>
              Patch item flags in{" "}
              <code className="rounded bg-muted px-1.5 py-0.5 text-xs">appearances.dat</code>{" "}
              using visual edits or a config file with{" "}
              <code className="rounded bg-muted px-1.5 py-0.5 text-xs">[[edit]]</code> blocks.
            </p>
            <ol className="list-decimal space-y-1 pl-5 text-sm">
              <li>Choose the source appearances file and optional output path.</li>
              <li>Define item changes in the visual editor or point at a config file.</li>
              <li>Use Apply in the footer to write the patched file.</li>
            </ol>
          </AlertDescription>
        </Alert>

        <Frame>
          <FrameHeader>
            <FrameTitle className="text-base">Files</FrameTitle>
            <FrameDescription>
              Select the appearances database to read and where to save the patched result.
            </FrameDescription>
          </FrameHeader>
          <FramePanel>
            <div className="grid gap-6 md:grid-cols-2">
              <FilePickerField
                id="appearances-file"
                label="Source appearances.dat"
                value={appearancesPath}
                onChange={setAppearancesPath}
                filters={[{ name: "Appearances", extensions: ["dat"] }]}
                disabled={loading}
              />
              <FilePickerField
                id="appearances-output"
                label="Output file (optional)"
                value={outputPath}
                onChange={setOutputPath}
                placeholder="Defaults to appearances.out.dat beside input"
                filters={[{ name: "DAT", extensions: ["dat"] }]}
                disabled={loading}
              />
            </div>
          </FramePanel>
        </Frame>

        <Frame>
          <FrameHeader className="gap-4 lg:flex-row lg:items-end lg:justify-between">
            <div className="space-y-1">
              <FrameTitle className="text-base">Item edits</FrameTitle>
              <FrameDescription>
                Default / On / Off matches config.toml booleans, including{" "}
                <code className="rounded bg-muted px-1 text-xs">unmove = false</code>.
              </FrameDescription>
            </div>
            <Tabs
              value={editorMode}
              onValueChange={handleEditorModeChange}
              className="w-full lg:w-auto"
            >
              <TabsList className="w-full lg:w-auto">
                <TabsTab value="ui">Visual editor</TabsTab>
                <TabsTab value="file">Config file</TabsTab>
              </TabsList>
            </Tabs>
          </FrameHeader>

          {editorMode === "ui" ? (
            <FramePanel className="overflow-hidden p-0">
              <AppearanceItemsEditor
                edits={config.appearanceEdits}
                selectedIndex={selectedIndex}
                onSelect={setSelectedIndex}
                onUpdate={updateAppearanceEdit}
                onRemove={(index) => {
                  removeAppearanceEdit(index);
                  setSelectedIndex((current) => Math.max(0, current - 1));
                }}
                onDuplicate={duplicateAppearanceEdit}
                onAdd={(entry) => {
                  addAppearanceEdit(entry);
                  setSelectedIndex(config.appearanceEdits.length);
                }}
                disabled={loading}
              />
            </FramePanel>
          ) : (
            <FramePanel className="space-y-5">
              <Alert variant="info">
                <AlertTitle>Using config.toml for item edits</AlertTitle>
                <AlertDescription>
                  The patch reads{" "}
                  <code className="rounded bg-muted px-1 text-xs">[[edit]]</code> blocks from the
                  file below. Load into the visual editor to preview or tweak entries.
                </AlertDescription>
              </Alert>

              <FilePickerField
                id="appearance-config-file"
                label="Config file"
                value={config.appearanceFilePath}
                onChange={setAppearanceFilePath}
                filters={[{ name: "TOML", extensions: ["toml"] }]}
                disabled={loading}
              />

              <div className="flex flex-wrap gap-2">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => void handleLoadEditsFile()}
                  disabled={loading}
                >
                  <RefreshCwIcon aria-hidden="true" />
                  Load into visual editor
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => void handleImportEditsFile()}
                  disabled={loading}
                >
                  <UploadIcon aria-hidden="true" />
                  Import file
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => void handleExportEdits()}
                  disabled={loading}
                >
                  <DownloadIcon aria-hidden="true" />
                  Export current edits
                </Button>
              </div>

              {config.appearanceEdits.length > 0 && (
                <p className="text-muted-foreground text-sm">
                  {config.appearanceEdits.length} item edit(s) available in memory from the last
                  import or UI session.
                </p>
              )}
            </FramePanel>
          )}
        </Frame>

        {(message || error || result) && (
          <section className="space-y-3">
            {message && (
              <Alert variant="success">
                <AlertTitle>Done</AlertTitle>
                <AlertDescription>{message}</AlertDescription>
              </Alert>
            )}
            {error && (
              <Alert variant="error">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            {result && (
              <Alert>
                <AlertTitle>Output written</AlertTitle>
                <AlertDescription>{result.output_path}</AlertDescription>
              </Alert>
            )}
          </section>
        )}

        <section className="space-y-3">
          <div>
            <h2 className="font-semibold text-base">Output log</h2>
            <p className="text-muted-foreground text-sm">
              Detailed messages from the appearances patch run.
            </p>
          </div>
          <LogViewer logs={result?.logs ?? []} className="min-h-72" />
        </section>
      </div>
    </PanelPage>
  );
};
