"use client";

import { useCallback, useEffect, useState } from "react";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import {
  AlertDialog,
  AlertDialogClose,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogPopup,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { isTauriEnvironment } from "@/lib/tauri/environment";

export const AppUpdater = () => {
  const [update, setUpdate] = useState<Update | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauriEnvironment()) {
      return;
    }

    let cancelled = false;

    const runUpdateCheck = async () => {
      try {
        const availableUpdate = await check();
        if (cancelled || !availableUpdate) {
          return;
        }

        setUpdate(availableUpdate);
        setDialogOpen(true);
      } catch {
        // Ignore update check failures (offline, missing release, dev builds).
      }
    };

    void runUpdateCheck();

    return () => {
      cancelled = true;
    };
  }, []);

  const handleInstallUpdate = useCallback(async () => {
    if (!update) {
      return;
    }

    setInstalling(true);
    setInstallError(null);

    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch (caught) {
      setInstallError(caught instanceof Error ? caught.message : String(caught));
      setInstalling(false);
    }
  }, [update]);

  if (!update) {
    return null;
  }

  return (
    <AlertDialog open={dialogOpen} onOpenChange={setDialogOpen}>
      <AlertDialogPopup>
        <AlertDialogHeader>
          <AlertDialogTitle>Update available</AlertDialogTitle>
          <AlertDialogDescription className="space-y-2 text-left">
            <span className="block">
              Version <strong>{update.version}</strong> is available. Install now to update Tibia
              Client Editor.
            </span>
            {update.body ? <span className="block whitespace-pre-wrap">{update.body}</span> : null}
            {installError ? (
              <span className="block text-destructive">Update failed: {installError}</span>
            ) : null}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogClose
            render={
              <Button type="button" variant="outline" disabled={installing}>
                Later
              </Button>
            }
          />
          <Button type="button" onClick={() => void handleInstallUpdate()} disabled={installing}>
            {installing ? <Spinner aria-hidden="true" /> : null}
            Install update
          </Button>
        </AlertDialogFooter>
      </AlertDialogPopup>
    </AlertDialog>
  );
};
