"use client";

import { useCallback, useState } from "react";
import { WrenchIcon } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { ConfigSourceBanner } from "@/components/app/config-source-banner";
import { DiagnosisSummary } from "@/components/app/diagnosis-summary";
import { FilePickerField } from "@/components/app/file-picker-field";
import { LogViewer } from "@/components/app/log-viewer";
import { PanelPage } from "@/components/app/panel-page";
import { useConfigStore } from "@/hooks/use-config-store";
import type { EditOutput } from "@/lib/models.types";
import { editClient } from "@/lib/tauri/commands";

export const EditPanel = () => {
  const config = useConfigStore();
  const [tibiaExe, setTibiaExe] = useState("");
  const [sourceExe, setSourceExe] = useState("");
  const [strictMode, setStrictMode] = useState(false);
  const [aggressiveMode, setAggressiveMode] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<EditOutput | null>(null);

  const handleEdit = useCallback(async () => {
    if (!tibiaExe) {
      setError("Select the Tibia client executable to patch.");
      return;
    }

    if (config.source === "file" && !config.filePath) {
      setError("File mode is enabled but no config.toml is selected. Open the Config tab first.");
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const output = await editClient({
        tibiaExe,
        config,
        sourceTibiaExe: sourceExe || undefined,
        strictClientCheck: strictMode,
        aggressiveClientCheck: aggressiveMode,
      });
      setResult(output);
      if (!output.success) {
        setError("Edit completed with errors. Review the logs below.");
      }
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [aggressiveMode, config, sourceExe, strictMode, tibiaExe]);

  return (
    <PanelPage
      footer={
        <div className="flex flex-wrap items-center justify-end gap-3">
          <Button
            type="button"
            size="lg"
            onClick={() => void handleEdit()}
            disabled={loading}
            data-loading={loading ? "" : undefined}
          >
            {loading ? <Spinner aria-hidden="true" /> : <WrenchIcon aria-hidden="true" />}
            Patch client
          </Button>
        </div>
      }
    >
      <div className="grid gap-6 px-6 py-6 lg:grid-cols-2 lg:px-8">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <WrenchIcon className="size-4" aria-hidden="true" />
              Patch Client
            </CardTitle>
            <CardDescription>
              Replace embedded URLs, sync config.ini, apply RSA key and BattlEye patches.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4">
            <ConfigSourceBanner config={config} />
            <FilePickerField
              id="edit-tibia-exe"
              label="Tibia executable"
              value={tibiaExe}
              onChange={setTibiaExe}
              filters={[{ name: "Executable", extensions: ["exe", ""] }]}
            />
            <FilePickerField
              id="edit-source-exe"
              label="Source executable (optional)"
              value={sourceExe}
              onChange={setSourceExe}
              placeholder="Defaults to client - original.exe beside target"
              filters={[{ name: "Executable", extensions: ["exe", ""] }]}
            />
            <div className="flex flex-wrap gap-6">
              <div className="flex items-center gap-2">
                <Switch
                  id="edit-strict"
                  checked={strictMode}
                  onCheckedChange={setStrictMode}
                  aria-label="Strict client-check mode"
                />
                <Label htmlFor="edit-strict">Strict mode</Label>
              </div>
              <div className="flex items-center gap-2">
                <Switch
                  id="edit-aggressive"
                  checked={aggressiveMode}
                  onCheckedChange={setAggressiveMode}
                  aria-label="Aggressive client-check mode"
                />
                <Label htmlFor="edit-aggressive">Aggressive mode</Label>
              </div>
            </div>
            {aggressiveMode && (
              <Alert variant="warning">
                <AlertTitle>Experimental aggressive mode</AlertTitle>
                <AlertDescription>
                  High-risk client-check signatures will be rewritten. Keep a backup and validate
                  manually.
                </AlertDescription>
              </Alert>
            )}
            {error && (
              <Alert variant="error">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>

        <div className="grid gap-4">
          {result?.diagnosis && (
            <DiagnosisSummary report={result.diagnosis} title="Post-patch diagnosis" />
          )}
          <LogViewer logs={result?.logs ?? []} className="min-h-72 lg:min-h-[32rem]" />
        </div>
      </div>
    </PanelPage>
  );
};
