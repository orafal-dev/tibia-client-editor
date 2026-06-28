"use client";

import { useMemo } from "react";
import {
  CopyIcon,
  PlusIcon,
  SparklesIcon,
  Trash2Icon,
} from "lucide-react";
import type { AppearanceEditEntry } from "@/lib/config/config.types";
import {
  APPEARANCE_BOOLEAN_FLAG_META,
  APPEARANCE_FLAG_GROUPS,
  APPEARANCE_OBJECT_FLAG_META,
  countActiveFlags,
  getBooleanFlagState,
  setBooleanFlagState,
  type AppearanceBooleanFlagKey,
  type AppearanceObjectFlagKey,
  type FlagTriState,
} from "@/lib/config/appearance-flags.types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectItem,
  SelectPopup,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";

type AppearanceItemsEditorProps = {
  edits: AppearanceEditEntry[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  onUpdate: (index: number, patch: Partial<AppearanceEditEntry>) => void;
  onRemove: (index: number) => void;
  onDuplicate: (index: number) => void;
  onAdd: (entry?: AppearanceEditEntry) => void;
  disabled?: boolean;
  compact?: boolean;
};

const FlagTriStateSelect = ({
  id,
  label,
  description,
  value,
  onChange,
  disabled,
}: {
  id: string;
  label: string;
  description: string;
  value: FlagTriState;
  onChange: (value: FlagTriState) => void;
  disabled?: boolean;
}) => (
  <Field className="gap-1.5">
    <FieldLabel htmlFor={id}>{label}</FieldLabel>
    <Select
      value={value}
      onValueChange={(next) => onChange(next as FlagTriState)}
      disabled={disabled}
    >
      <SelectTrigger id={id} aria-label={label} size="sm">
        <SelectValue />
      </SelectTrigger>
      <SelectPopup>
        <SelectItem value="unset">Default</SelectItem>
        <SelectItem value="true">On</SelectItem>
        <SelectItem value="false">Off</SelectItem>
      </SelectPopup>
    </Select>
    <FieldDescription>{description}</FieldDescription>
  </Field>
);

const SelectedItemEditor = ({
  edit,
  index,
  onUpdate,
  onRemove,
  onDuplicate,
  disabled,
  spacious = false,
}: {
  edit: AppearanceEditEntry;
  index: number;
  onUpdate: (index: number, patch: Partial<AppearanceEditEntry>) => void;
  onRemove: (index: number) => void;
  onDuplicate: (index: number) => void;
  disabled?: boolean;
  spacious?: boolean;
}) => {
  const handleBooleanChange = (key: AppearanceBooleanFlagKey, state: FlagTriState) => {
    onUpdate(index, {
      booleanFlags: setBooleanFlagState(edit.booleanFlags, key, state),
    });
  };

  const handleObjectChange = (key: AppearanceObjectFlagKey, enabled: boolean) => {
    const next = { ...edit.objectFlags };
    if (enabled) {
      next[key] = true;
    } else {
      delete next[key];
    }
    onUpdate(index, { objectFlags: next });
  };

  const flagGroups = (
    <div className="grid gap-4">
      {APPEARANCE_FLAG_GROUPS.map((group) => (
        <Collapsible
          key={group.id}
          defaultOpen={group.id === "common"}
          className="rounded-xl border"
        >
          <CollapsibleTrigger className="flex w-full items-center justify-between px-4 py-3 text-left">
            <div>
              <div className="font-medium">{group.title}</div>
              <div className="text-muted-foreground text-sm">{group.description}</div>
            </div>
            <Badge variant="outline">
              {group.booleanFlags.filter((key) => key in edit.booleanFlags).length +
                (group.objectFlags?.filter((key) => edit.objectFlags[key]).length ?? 0)}{" "}
              set
            </Badge>
          </CollapsibleTrigger>
          <CollapsibleContent className="grid gap-4 border-t px-4 py-4 sm:grid-cols-2 xl:grid-cols-3">
            {group.booleanFlags.map((key) => (
              <FlagTriStateSelect
                key={key}
                id={`${index}-${key}`}
                label={APPEARANCE_BOOLEAN_FLAG_META[key].label}
                description={APPEARANCE_BOOLEAN_FLAG_META[key].description}
                value={getBooleanFlagState(edit.booleanFlags, key)}
                onChange={(state) => handleBooleanChange(key, state)}
                disabled={disabled}
              />
            ))}
            {group.objectFlags?.map((key) => (
              <div
                key={key}
                className="flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
              >
                <div>
                  <Label htmlFor={`${index}-obj-${key}`} className="font-medium">
                    {APPEARANCE_OBJECT_FLAG_META[key].label}
                  </Label>
                  <p className="text-muted-foreground text-xs">
                    {APPEARANCE_OBJECT_FLAG_META[key].description}
                  </p>
                </div>
                <Switch
                  id={`${index}-obj-${key}`}
                  checked={Boolean(edit.objectFlags[key])}
                  onCheckedChange={(checked) => handleObjectChange(key, checked)}
                  disabled={disabled}
                  aria-label={APPEARANCE_OBJECT_FLAG_META[key].label}
                />
              </div>
            ))}
          </CollapsibleContent>
        </Collapsible>
      ))}
    </div>
  );

  return (
    <div className={cn("flex min-h-0 flex-col", spacious ? "h-full" : "grid gap-4")}>
      <div className={cn(spacious ? "space-y-5 border-b p-6" : "grid gap-4")}>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h3 className={cn("font-medium", spacious && "text-base font-semibold")}>
              Item details
            </h3>
            <p className="text-muted-foreground text-sm">
              Maps to a <code className="rounded bg-muted px-1">[[edit]]</code> block in
              config.toml. Only non-default flags are written to the output file.
            </p>
          </div>
          <div className="flex shrink-0 gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => onDuplicate(index)}
              disabled={disabled}
            >
              <CopyIcon aria-hidden="true" />
              Duplicate
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => onRemove(index)}
              disabled={disabled}
            >
              <Trash2Icon aria-hidden="true" />
              Remove
            </Button>
          </div>
        </div>

        <div className="grid gap-4 sm:grid-cols-2">
          <Field>
            <FieldLabel htmlFor={`appearance-id-${index}`}>Item ID</FieldLabel>
            <Input
              id={`appearance-id-${index}`}
              value={edit.id}
              onChange={(event) => onUpdate(index, { id: event.target.value })}
              disabled={disabled}
              placeholder="24964"
              aria-label="Item ID"
            />
            <FieldDescription>
              Client/server item type ID to patch in appearances.dat.
            </FieldDescription>
          </Field>
          <Field>
            <FieldLabel htmlFor={`appearance-label-${index}`}>Label (optional)</FieldLabel>
            <Input
              id={`appearance-label-${index}`}
              value={edit.label}
              onChange={(event) => onUpdate(index, { label: event.target.value })}
              disabled={disabled}
              placeholder="Imbuing Crystal"
              aria-label="Item label"
            />
            <FieldDescription>Used in the UI and exported as a TOML comment.</FieldDescription>
          </Field>
        </div>
      </div>

      {spacious ? (
        <ScrollArea className="min-h-0 flex-1">
          <div className="space-y-4 p-6">{flagGroups}</div>
        </ScrollArea>
      ) : (
        flagGroups
      )}
    </div>
  );
};

export const AppearanceItemsEditor = ({
  edits,
  selectedIndex,
  onSelect,
  onUpdate,
  onRemove,
  onDuplicate,
  onAdd,
  disabled = false,
  compact = false,
}: AppearanceItemsEditorProps) => {
  const safeIndex = edits.length === 0 ? 0 : Math.min(selectedIndex, edits.length - 1);
  const selectedEdit = edits.length > 0 ? edits[safeIndex] : undefined;

  const listClassName = useMemo(
    () => cn("grid gap-2", compact ? "max-h-56 overflow-y-auto pr-1" : ""),
    [compact],
  );

  if (compact) {
    return (
      <div className="grid gap-4">
        <div className="grid gap-3 rounded-xl border p-3">
          <div className="flex items-center justify-between gap-2">
            <div>
              <div className="font-medium">Items to edit</div>
              <p className="text-muted-foreground text-sm">{edits.length} configured</p>
            </div>
            <Button type="button" size="sm" onClick={() => onAdd()} disabled={disabled}>
              <PlusIcon aria-hidden="true" />
              Add
            </Button>
          </div>

          <div className={listClassName}>
            {edits.length === 0 ? (
              <div className="rounded-lg border border-dashed px-3 py-6 text-center text-muted-foreground text-sm">
                No items configured yet. Add one to start editing appearance flags.
              </div>
            ) : (
              edits.map((edit, index) => (
                <button
                  key={`${index}-${edit.id}`}
                  type="button"
                  className={cn(
                    "flex w-full items-start gap-3 rounded-lg border px-3 py-2 text-left transition-colors",
                    safeIndex === index
                      ? "border-primary bg-primary/5"
                      : "hover:bg-muted/50",
                  )}
                  onClick={() => onSelect(index)}
                  aria-label={`Edit item ${edit.label || edit.id || index + 1}`}
                >
                  <SparklesIcon
                    className="mt-0.5 size-4 shrink-0 text-muted-foreground"
                    aria-hidden="true"
                  />
                  <div className="min-w-0 flex-1">
                    <div className="truncate font-medium">
                      {edit.label.trim() || `Item ${edit.id.trim() || index + 1}`}
                    </div>
                    <div className="truncate text-muted-foreground text-xs">
                      ID {edit.id.trim() || "—"} · {countActiveFlags(edit)} flags
                    </div>
                  </div>
                </button>
              ))
            )}
          </div>
        </div>

        <div className="rounded-xl border p-4">
          {selectedEdit ? (
            <SelectedItemEditor
              edit={selectedEdit}
              index={safeIndex}
              onUpdate={onUpdate}
              onRemove={onRemove}
              onDuplicate={onDuplicate}
              disabled={disabled}
            />
          ) : (
            <div className="py-16 text-center text-muted-foreground text-sm">
              Select or add an item to configure its appearance flags.
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="grid min-h-[640px] w-full lg:grid-cols-[minmax(300px,340px)_minmax(0,1fr)]">
      <aside className="flex flex-col overflow-hidden border-b bg-muted/15 lg:border-r lg:border-b-0">
        <div className="space-y-4 border-b p-5">
          <div>
            <h3 className="font-medium text-sm">Items to edit</h3>
            <p className="mt-1 text-muted-foreground text-xs leading-relaxed">
              Each entry is one item ID. Select an item to configure flags on the
              right.
            </p>
          </div>

          <div className="flex items-center justify-between gap-2">
            <p className="text-muted-foreground text-sm">{edits.length} configured</p>
            <Button type="button" size="sm" onClick={() => onAdd()} disabled={disabled}>
              <PlusIcon aria-hidden="true" />
              Add item
            </Button>
          </div>
        </div>

        <ScrollArea className="min-h-[320px] flex-1">
          <div className="space-y-2 p-4">
            {edits.length === 0 ? (
              <div className="rounded-xl border border-dashed bg-background px-4 py-8 text-center">
                <p className="text-muted-foreground text-sm">No items yet</p>
                <p className="mt-1 text-muted-foreground text-xs">
                  Use Add item to create your first entry.
                </p>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="mt-4"
                  onClick={() => onAdd()}
                  disabled={disabled}
                >
                  Add your first item
                </Button>
              </div>
            ) : (
              edits.map((edit, index) => (
                <button
                  key={`${index}-${edit.id}`}
                  type="button"
                  className={cn(
                    "flex w-full items-start gap-3 rounded-xl border px-3 py-3 text-left transition-colors",
                    safeIndex === index
                      ? "border-primary bg-primary/5 shadow-xs"
                      : "border-transparent bg-background hover:border-border",
                  )}
                  onClick={() => onSelect(index)}
                  aria-label={`Edit item ${edit.label || edit.id || index + 1}`}
                >
                  <SparklesIcon
                    className="mt-0.5 size-4 shrink-0 text-muted-foreground"
                    aria-hidden="true"
                  />
                  <div className="min-w-0 flex-1">
                    <div className="truncate font-medium text-sm">
                      {edit.label.trim() || `Item ${edit.id.trim() || index + 1}`}
                    </div>
                    <div className="truncate text-muted-foreground text-xs">
                      ID {edit.id.trim() || "—"} · {countActiveFlags(edit)} flags
                    </div>
                  </div>
                </button>
              ))
            )}
          </div>
        </ScrollArea>
      </aside>

      <section className="flex min-h-[640px] min-w-0 flex-col bg-background">
        {selectedEdit ? (
          <SelectedItemEditor
            edit={selectedEdit}
            index={safeIndex}
            onUpdate={onUpdate}
            onRemove={onRemove}
            onDuplicate={onDuplicate}
            disabled={disabled}
            spacious
          />
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 p-10 text-center">
            <p className="font-medium text-sm">Select an item to edit</p>
            <p className="max-w-md text-muted-foreground text-sm leading-relaxed">
              Choose an item from the list on the left, or add one to configure ID,
              label, and appearance flags.
            </p>
            <Button type="button" variant="outline" onClick={() => onAdd()} disabled={disabled}>
              Add your first item
            </Button>
          </div>
        )}
      </section>
    </div>
  );
};
