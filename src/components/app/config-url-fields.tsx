"use client";

import type { ClientConfig } from "@/lib/models.types";
import { URL_PROPERTY_LABELS } from "@/lib/models.types";
import type { UrlConfigGroup } from "@/lib/config/config.types";
import { URL_CONFIG_GROUPS } from "@/lib/config/config.types";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Fieldset, FieldsetLegend } from "@/components/ui/fieldset";
import { Input } from "@/components/ui/input";

type ConfigUrlFieldsProps = {
  urls: ClientConfig;
  onChange: (key: keyof ClientConfig, value: string) => void;
  disabled?: boolean;
};

export const ConfigUrlFields = ({ urls, onChange, disabled = false }: ConfigUrlFieldsProps) => (
  <div className="grid gap-6">
    {URL_CONFIG_GROUPS.map((group: UrlConfigGroup) => (
      <Fieldset key={group.id} className="rounded-xl border p-4">
        <FieldsetLegend>{group.title}</FieldsetLegend>
        <p className="mb-4 text-sm text-muted-foreground">{group.description}</p>
        <div className="grid gap-4 sm:grid-cols-2">
          {group.keys.map((key) => (
            <Field key={key}>
              <FieldLabel htmlFor={`url-${key}`}>{URL_PROPERTY_LABELS[key]}</FieldLabel>
              <Input
                id={`url-${key}`}
                value={urls[key]}
                onChange={(event) => onChange(key, event.target.value)}
                disabled={disabled}
                aria-label={URL_PROPERTY_LABELS[key]}
              />
              <FieldDescription>{key}</FieldDescription>
            </Field>
          ))}
        </div>
      </Fieldset>
    ))}
  </div>
);
