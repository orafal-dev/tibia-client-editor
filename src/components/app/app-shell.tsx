"use client";

import {
  AppleIcon,
  PackageIcon,
  Settings2Icon,
  SparklesIcon,
  StethoscopeIcon,
  WrenchIcon,
} from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AppearancesPanel } from "@/components/app/appearances-panel";
import { ConfigWorkspace } from "@/components/app/config-workspace";
import { DiagnosePanel } from "@/components/app/diagnose-panel";
import { EditPanel } from "@/components/app/edit-panel";
import { RepackPanel } from "@/components/app/repack-panel";
import { Win2MacPanel } from "@/components/app/win2mac-panel";
import { ThemeToggle } from "@/components/app/theme-toggle";
import { useHydrateConfigStore } from "@/hooks/use-config-store";
import { Spinner } from "@/components/ui/spinner";

export const AppShell = () => {
  const loaded = useHydrateConfigStore();

  if (!loaded) {
    return (
      <div className="flex h-full min-h-0 items-center justify-center">
        <Spinner aria-label="Loading configuration" />
      </div>
    );
  }

  return (
    <Tabs defaultValue="config" className="flex h-full min-h-0 flex-col gap-0 overflow-hidden bg-background">
      <header className="shrink-0 border-b bg-card/50">
        <div className="flex items-start justify-between gap-4 px-6 pt-4 lg:px-8">
          <div className="min-w-0 flex-1 space-y-1">
            <h1 className="text-xl font-semibold tracking-tight">Tibia Client Editor</h1>
            <p className="text-sm text-muted-foreground">
              Patch Tibia 11+ clients with custom URLs, RSA keys, BattlEye bypasses, and appearance edits.
            </p>
          </div>
          <ThemeToggle />
        </div>

        <div className="px-6 pb-4 pt-4 lg:px-8">
          <TabsList className="flex h-auto w-full flex-wrap justify-start gap-1">
            <TabsTrigger value="config" className="gap-2">
              <Settings2Icon className="size-4" aria-hidden="true" />
              Config
            </TabsTrigger>
            <TabsTrigger value="edit" className="gap-2">
              <WrenchIcon className="size-4" aria-hidden="true" />
              Patch
            </TabsTrigger>
            <TabsTrigger value="diagnose" className="gap-2">
              <StethoscopeIcon className="size-4" aria-hidden="true" />
              Diagnose
            </TabsTrigger>
            <TabsTrigger value="repack" className="gap-2">
              <PackageIcon className="size-4" aria-hidden="true" />
              Repack
            </TabsTrigger>
            <TabsTrigger value="appearances" className="gap-2">
              <SparklesIcon className="size-4" aria-hidden="true" />
              Appearances
            </TabsTrigger>
            <TabsTrigger value="win2mac" className="gap-2">
              <AppleIcon className="size-4" aria-hidden="true" />
              Win → Mac
            </TabsTrigger>
          </TabsList>
        </div>
      </header>

      <TabsContent value="config" className="mt-0 flex min-h-0 flex-1 flex-col overflow-hidden">
        <ConfigWorkspace />
      </TabsContent>
      <TabsContent value="edit" className="mt-0 flex min-h-0 flex-1 flex-col overflow-hidden">
        <EditPanel />
      </TabsContent>
      <TabsContent value="diagnose" className="mt-0 flex min-h-0 flex-1 flex-col overflow-hidden">
        <DiagnosePanel />
      </TabsContent>
      <TabsContent value="repack" className="mt-0 flex min-h-0 flex-1 flex-col overflow-hidden">
        <RepackPanel />
      </TabsContent>
      <TabsContent value="appearances" className="mt-0 flex min-h-0 flex-1 flex-col overflow-hidden">
        <AppearancesPanel />
      </TabsContent>
      <TabsContent value="win2mac" className="mt-0 flex min-h-0 flex-1 flex-col overflow-hidden">
        <Win2MacPanel />
      </TabsContent>
    </Tabs>
  );
};
