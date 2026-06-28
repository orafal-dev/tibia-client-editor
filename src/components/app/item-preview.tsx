"use client";

import { useState } from "react";
import { SparklesIcon } from "lucide-react";
import { getItemPreviewUrl } from "@/lib/item-preview";
import { cn } from "@/lib/utils";

type ItemPreviewProps = {
  itemId: string;
  className?: string;
};

export const ItemPreview = ({ itemId, className }: ItemPreviewProps) => {
  const trimmedId = itemId.trim();
  const previewUrl = getItemPreviewUrl(itemId);
  const [hasError, setHasError] = useState(false);

  if (!previewUrl || hasError) {
    return (
      <div
        className={cn(
          "flex size-8 shrink-0 items-center justify-center rounded-md border bg-muted/40",
          className,
        )}
        aria-label={
          trimmedId ? `Item ${trimmedId} preview unavailable` : "Item preview unavailable"
        }
        role="img"
      >
        <SparklesIcon className="size-4 text-muted-foreground" aria-hidden="true" />
      </div>
    );
  }

  return (
    <img
      key={previewUrl}
      src={previewUrl}
      alt={`Item ${trimmedId} preview`}
      width={32}
      height={32}
      className={cn("size-8 shrink-0 object-contain", className)}
      onError={() => setHasError(true)}
    />
  );
};
