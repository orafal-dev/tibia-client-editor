"use client";

import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

type LogViewerProps = {
  logs: string[];
  className?: string;
  emptyMessage?: string;
};

const getLogClassName = (line: string): string => {
  if (line.startsWith("[ERROR]")) return "text-destructive";
  if (line.startsWith("[WARN]")) return "text-warning";
  if (line.startsWith("[PATCH]")) return "text-success";
  if (line.startsWith("[INFO]")) return "text-muted-foreground";
  return "text-foreground";
};

export const LogViewer = ({
  logs,
  className,
  emptyMessage = "Run an action to see output logs here.",
}: LogViewerProps) => (
  <ScrollArea className={cn("h-64 rounded-lg border bg-muted/30", className)}>
    <pre className="p-4 font-mono text-xs leading-relaxed">
      {logs.length === 0 ? (
        <span className="text-muted-foreground">{emptyMessage}</span>
      ) : (
        logs.map((line, index) => (
          <div key={`${index}-${line.slice(0, 24)}`} className={getLogClassName(line)}>
            {line}
          </div>
        ))
      )}
    </pre>
  </ScrollArea>
);
