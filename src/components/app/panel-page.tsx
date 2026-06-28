"use client";

import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

type PanelPageProps = {
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
};

export const PanelPage = ({ children, footer, className }: PanelPageProps) => (
  <div className={cn("flex h-full min-h-0 flex-1 flex-col", className)}>
    <div className="min-h-0 flex-1 overflow-y-auto overscroll-y-contain">{children}</div>
    {footer ? (
      <footer className="shrink-0 border-t bg-background/95 px-6 py-4 backdrop-blur-sm lg:px-8">
        {footer}
      </footer>
    ) : null}
  </div>
);
