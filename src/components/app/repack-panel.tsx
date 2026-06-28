"use client";

import { useCallback, useState } from "react";
import { PackageIcon } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectItem,
  SelectPopup,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Spinner } from "@/components/ui/spinner";
import { FilePickerField } from "@/components/app/file-picker-field";
import { LogViewer } from "@/components/app/log-viewer";
import { PanelPage } from "@/components/app/panel-page";
import type { RepackPlatform, RepackResult } from "@/lib/models.types";
import { repackClient } from "@/lib/tauri/commands";

const REPACK_PLATFORM_ITEMS = [
  { value: "windows", label: "Windows" },
  { value: "mac", label: "macOS" },
  { value: "linux", label: "Linux" },
] as const;

export const RepackPanel = () => {
  const [src, setSrc] = useState("");
  const [dst, setDst] = useState("");
  const [platform, setPlatform] = useState<RepackPlatform>("windows");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<RepackResult | null>(null);

  const handleRepack = useCallback(async () => {
    if (!src || !dst) {
      setError("Select both source client folder and output folder.");
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const output = await repackClient({ src, dst, platform });
      setResult(output);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setLoading(false);
    }
  }, [dst, platform, src]);

  return (
    <PanelPage
      footer={
        <div className="flex flex-wrap items-center justify-end gap-3">
          <Button
            type="button"
            size="lg"
            onClick={() => void handleRepack()}
            disabled={loading}
            data-loading={loading ? "" : undefined}
          >
            {loading ? <Spinner aria-hidden="true" /> : <PackageIcon aria-hidden="true" />}
            Repack client
          </Button>
        </div>
      }
    >
      <div className="grid gap-6 px-6 py-6 lg:grid-cols-2 lg:px-8">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <PackageIcon className="size-4" aria-hidden="true" />
              Repack Client
            </CardTitle>
            <CardDescription>
              Repack a Tibia client folder for use with slender-launcher. Requires client.json and
              assets.json.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4">
            <FilePickerField
              id="repack-src"
              label="Source client folder"
              value={src}
              onChange={setSrc}
              mode="directory"
            />
            <FilePickerField
              id="repack-dst"
              label="Output folder"
              value={dst}
              onChange={setDst}
              mode="directory"
            />
            <div className="grid gap-2">
              <Label htmlFor="repack-platform">Platform</Label>
              <Select
                items={[...REPACK_PLATFORM_ITEMS]}
                value={platform}
                onValueChange={(value) => setPlatform(value as RepackPlatform)}
              >
                <SelectTrigger id="repack-platform" aria-label="Repack platform">
                  <SelectValue />
                </SelectTrigger>
                <SelectPopup>
                  <SelectItem value="windows">Windows</SelectItem>
                  <SelectItem value="mac">macOS</SelectItem>
                  <SelectItem value="linux">Linux</SelectItem>
                </SelectPopup>
              </Select>
            </div>
            {error && (
              <Alert variant="error">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            {result && (
              <Alert>
                <AlertTitle>Repack complete</AlertTitle>
                <AlertDescription>
                  {result.client_files} client files, {result.assets_files} asset files — revision{" "}
                  {result.revision}
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
