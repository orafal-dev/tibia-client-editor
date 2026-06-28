"use client";

import { useCallback, useState } from "react";
import { AppleIcon } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { FilePickerField } from "@/components/app/file-picker-field";
import { LogViewer } from "@/components/app/log-viewer";
import { PanelPage } from "@/components/app/panel-page";
import type { Win2MacResult } from "@/lib/models.types";
import { win2macAssets } from "@/lib/tauri/commands";

export const Win2MacPanel = () => {
  const [src, setSrc] = useState("");
  const [dst, setDst] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<Win2MacResult | null>(null);

  const handleConvert = useCallback(async () => {
    if (!src || !dst) {
      setError("Select source and destination assets.json paths.");
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const output = await win2macAssets({ src, dst });
      setResult(output);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [dst, src]);

  return (
    <PanelPage
      footer={
        <div className="flex flex-wrap items-center justify-end gap-3">
          <Button
            type="button"
            size="lg"
            onClick={() => void handleConvert()}
            disabled={loading}
            data-loading={loading ? "" : undefined}
          >
            {loading ? <Spinner aria-hidden="true" /> : <AppleIcon aria-hidden="true" />}
            Convert manifest
          </Button>
        </div>
      }
    >
      <div className="grid gap-6 px-6 py-6 lg:grid-cols-2 lg:px-8">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <AppleIcon className="size-4" aria-hidden="true" />
              Windows → macOS Assets
            </CardTitle>
            <CardDescription>
              Convert a Windows assets.json manifest to macOS by prefixing paths with
              Contents/Resources/.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4">
            <FilePickerField
              id="win2mac-src"
              label="Windows assets.json"
              value={src}
              onChange={setSrc}
              filters={[{ name: "JSON", extensions: ["json"] }]}
            />
            <FilePickerField
              id="win2mac-dst"
              label="Output assets.json"
              value={dst}
              onChange={setDst}
              filters={[{ name: "JSON", extensions: ["json"] }]}
            />
            {error && (
              <Alert variant="error">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            {result && (
              <Alert>
                <AlertTitle>Conversion complete</AlertTitle>
                <AlertDescription>
                  Updated {result.files_updated} file path(s) → {result.output_path}
                </AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>
        <LogViewer logs={result?.logs ?? []} className="min-h-72 lg:min-h-[28rem]" />
      </div>
    </PanelPage>
  );
};
