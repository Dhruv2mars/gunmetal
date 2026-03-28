import type { Metadata } from "next";
import { Barlow_Semi_Condensed, IBM_Plex_Mono, Teko } from "next/font/google";
import "./globals.css";

const display = Teko({
  variable: "--font-display",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

const body = Barlow_Semi_Condensed({
  variable: "--font-body",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

const mono = IBM_Plex_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  weight: ["400", "500", "600"],
});

export const metadata: Metadata = {
  metadataBase: new URL("https://gunmetal.vercel.app"),
  title: {
    default: "Gunmetal",
    template: "%s | Gunmetal",
  },
  description:
    "Gunmetal is a local-first AI switchboard with a hosted landing page, a local browser UI, a TUI, a CLI, and one fast local API.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${display.variable} ${body.variable} ${mono.variable}`}
    >
      <body>{children}</body>
    </html>
  );
}
