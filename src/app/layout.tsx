import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { ThemeProvider } from "@/components/providers/theme-provider";
import { cn } from "@/lib/utils";

const inter = Inter({ subsets: ["latin"], variable: "--font-sans" });
const interHeading = Inter({ subsets: ["latin"], variable: "--font-heading" });

export const metadata: Metadata = {
  title: "Tibia Client Editor",
  description:
    "Patch Tibia 11+ clients with custom OTServ URLs, RSA keys, BattlEye bypasses, and appearance edits.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={cn(
        "dark h-dvh antialiased",
        inter.variable,
        interHeading.variable,
        "font-sans",
      )}
    >
      <body className="relative h-dvh overflow-hidden">
        <ThemeProvider>
          <div className="isolate flex h-full min-h-0 flex-col overflow-hidden">{children}</div>
        </ThemeProvider>
      </body>
    </html>
  );
}
