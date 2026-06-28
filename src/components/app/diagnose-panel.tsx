"use client";

import { useCallback, useState } from "react";
import { StethoscopeIcon } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { DiagnosisSummary } from "@/components/app/diagnosis-summary";
import { FilePickerField } from "@/components/app/file-picker-field";
import { LogViewer } from "@/components/app/log-viewer";
import { PanelPage } from "@/components/app/panel-page";
import type { DiagnoseOutput } from "@/lib/models.types";
import { diagnoseClient } from "@/lib/tauri/commands";

export const DiagnosePanel = () => {
  const [tibiaExe, setTibiaExe] = useState("");
  const [compareWith, setCompareWith] = useState("");
  const [strictMode, setStrictMode] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<DiagnoseOutput | null>(null);

  const handleDiagnose = useCallback(async () => {
    if (!tibiaExe) {
      setError("Select a Tibia executable to diagnose.");
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const output = await diagnoseClient({
        tibiaExe,
        compareWith: compareWith || undefined,
        strictClientCheck: strictMode,
      });
      setResult(output);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [compareWith, strictMode, tibiaExe]);

  return (
    <PanelPage
      footer={
        <div className="flex flex-wrap items-center justify-end gap-3">
          <Button
            type="button"
            size="lg"
            onClick={() => void handleDiagnose()}
            disabled={loading}
            data-loading={loading ? "" : undefined}
          >
            {loading ? <Spinner aria-hidden="true" /> : <StethoscopeIcon aria-hidden="true" />}
            Run diagnosis
          </Button>
        </div>
      }
    >
      <div className="grid gap-6 px-6 py-6 lg:px-8">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <StethoscopeIcon className="size-4" aria-hidden="true" />
              Diagnose Client
            </CardTitle>
            <CardDescription>
              Inspect BattlEye signatures, client-check indicators, and compatibility without
              modifying files.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 lg:grid-cols-2">
            <FilePickerField
              id="diagnose-tibia-exe"
              label="Tibia executable"
              value={tibiaExe}
              onChange={setTibiaExe}
              filters={[{ name: "Executable", extensions: ["exe", ""] }]}
            />
            <FilePickerField
              id="diagnose-compare"
              label="Compare with (optional)"
              value={compareWith}
              onChange={setCompareWith}
              filters={[{ name: "Executable", extensions: ["exe", ""] }]}
            />
            <div className="flex items-center gap-2 lg:col-span-2">
              <Switch
                id="diagnose-strict"
                checked={strictMode}
                onCheckedChange={setStrictMode}
                aria-label="Strict diagnosis mode"
              />
              <Label htmlFor="diagnose-strict">Fail on PARTIAL, WARNING, or UNSUPPORTED</Label>
            </div>
            {error && (
              <Alert variant="error" className="lg:col-span-2">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>

        {result?.target && <DiagnosisSummary report={result.target} title="Target client" />}
        {result?.baseline && <DiagnosisSummary report={result.baseline} title="Baseline client" />}
        <LogViewer
          logs={[...(result?.logs ?? []), ...(result?.comparison_logs ?? [])]}
          className="min-h-96"
        />
      </div>
    </PanelPage>
  );
};
