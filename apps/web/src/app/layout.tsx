import type { Metadata } from "next";
import { Inter, JetBrains_Mono, DM_Sans } from "next/font/google";
import "./globals.css";
import { ThemeProvider } from "@/components/providers/ThemeProvider";
import { Navbar } from "@/components/layout/Navbar";
import { Footer } from "@/components/layout/Footer";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  display: "swap",
  weight: ["400", "500", "600", "700", "800"],
});

const dmSans = DM_Sans({
  variable: "--font-matter",
  subsets: ["latin"],
  display: "swap",
  weight: ["400", "500"],
});

const jetbrains = JetBrains_Mono({
  variable: "--font-jetbrains",
  subsets: ["latin"],
  display: "swap",
  weight: ["400", "500"],
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
    <html lang="en" data-theme="dark">
      <body
        className={`${inter.variable} ${jetbrains.variable} ${dmSans.variable}`}
        style={{ colorScheme: "dark" }}
      >
        <ThemeProvider>
          <div className="min-h-screen flex flex-col bg-[var(--bg)] text-[var(--text)] selection:bg-[#faf9f6] selection:text-[#1a1a19] relative overflow-hidden">
            <Navbar />
            <main className="relative z-10 flex-1 flex flex-col">
              {children}
            </main>
            <Footer />
          </div>
        </ThemeProvider>
      </body>
    </html>
  );
}
