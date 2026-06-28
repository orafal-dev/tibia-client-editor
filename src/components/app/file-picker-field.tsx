"use client";

import { useCallback } from "react";
import { FolderOpenIcon, FileIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { pickDirectory, pickFile } from "@/lib/tauri/commands";

type FilePickerFieldProps = {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  mode?: "file" | "directory";
  filters?: { name: string; extensions: string[] }[];
  disabled?: boolean;
};

export const FilePickerField = ({
  id,
  label,
  value,
  onChange,
  placeholder = "Select a path…",
  mode = "file",
  filters,
  disabled = false,
}: FilePickerFieldProps) => {
  const handleBrowse = useCallback(async () => {
    const selected =
      mode === "directory"
        ? await pickDirectory(`Select ${label}`)
        : await pickFile(`Select ${label}`, filters);

    if (typeof selected === "string") {
      onChange(selected);
    }
  }, [filters, label, mode, onChange]);

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLButtonElement>) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        void handleBrowse();
      }
    },
    [handleBrowse],
  );

  return (
    <div className="grid gap-2">
      <Label htmlFor={id}>{label}</Label>
      <div className="flex gap-2">
        <Input
          id={id}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          placeholder={placeholder}
          disabled={disabled}
          aria-label={label}
        />
        <Button
          type="button"
          variant="outline"
          size="icon"
          onClick={() => void handleBrowse()}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          aria-label={`Browse for ${label}`}
          tabIndex={0}
        >
          {mode === "directory" ? (
            <FolderOpenIcon aria-hidden="true" />
          ) : (
            <FileIcon aria-hidden="true" />
          )}
        </Button>
      </div>
    </div>
  );
};
