import { Badge } from "@/components/ui/badge";
import type { AppConfigState } from "@/lib/config/config.types";

type ConfigSourceBannerProps = {
  config: AppConfigState;
};

export const ConfigSourceBanner = ({ config }: ConfigSourceBannerProps) => (
  <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-muted/30 px-3 py-2 text-sm">
    <span className="text-muted-foreground">Active configuration:</span>
    {config.source === "file" ? (
      <>
        <Badge variant="info">config.toml</Badge>
        <span className="truncate font-mono text-xs">{config.filePath || "No file selected"}</span>
      </>
    ) : (
      <>
        <Badge variant="success">UI</Badge>
        <span className="text-muted-foreground">{config.baseUrl}</span>
      </>
    )}
  </div>
);
