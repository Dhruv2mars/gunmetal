import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { ThemeProvider } from "@/components/providers/ThemeProvider";
import { Navbar } from "@/components/layout/Navbar";
import { Footer } from "@/components/layout/Footer";

const geist = Geist({
  variable: "--font-geist",
  subsets: ["latin"],
  display: "swap",
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  metadataBase: new URL("https://gunmetalapp.vercel.app"),
  title: {
    default: "Gunmetal",
    template: "%s | Gunmetal",
  },
  description:
    "One local API. Every AI provider. Gunmetal turns your AI subscriptions into a unified endpoint.",
  keywords: [
    "AI",
    "inference",
    "local API",
    "OpenAI compatible",
    "AI gateway",
    "model routing",
  ],
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://gunmetalapp.vercel.app",
    siteName: "Gunmetal",
    title: "Gunmetal — One Local API. Every AI Provider.",
    description:
      "Gunmetal turns your AI subscriptions into a unified endpoint. Route, control, and observe every request.",
  },
  twitter: {
    card: "summary_large_image",
    title: "Gunmetal — One Local API. Every AI Provider.",
    description:
      "Gunmetal turns your AI subscriptions into a unified endpoint. Route, control, and observe every request.",
  },
  icons: {
    icon: "/icon.svg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" data-theme="dark" data-scroll-behavior="smooth">
      <body
        className={`${geist.variable} ${geistMono.variable}`}
        style={{ colorScheme: "dark" }}
      >
        <ThemeProvider>
          <div className="relative flex min-h-[100dvh] flex-col overflow-hidden bg-[var(--bg)] text-[var(--text)] selection:bg-[#f2ead7] selection:text-[#171716]">
            <a
              href="#main"
              className="sr-only focus:not-sr-only focus:fixed focus:top-3 focus:left-3 focus:z-[200] focus:rounded-lg focus:bg-[rgba(14,14,13,0.95)] focus:px-3 focus:py-2 focus:text-[14px] focus:text-[var(--text)] focus:outline-none focus-visible:ring-2 focus-visible:ring-[rgba(250,249,246,0.18)]"
            >
              Skip to content
            </a>
            <Navbar />
            <main id="main" className="relative z-10 flex flex-1 flex-col">
              {children}
            </main>
            <Footer />
          </div>
        </ThemeProvider>
      </body>
    </html>
  );
}
