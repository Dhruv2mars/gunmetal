"use client";

import { useEffect, useState } from "react";
import { Navbar } from "@/components/layout/Navbar";
import { Footer } from "@/components/layout/Footer";

export default function HomePage() {
  // Add scroll listener for subtle parallax/fade effects
  const [scrollY, setScrollY] = useState(0);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrollY(window.scrollY);
    };
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const handleCopy = () => {
    navigator.clipboard.writeText("npm i -g @dhruv2mars/gunmetal");
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="min-h-screen flex flex-col bg-[var(--bg)] text-[var(--text)] selection:bg-[#faf9f6] selection:text-[#1a1a19] relative overflow-hidden">
      <Navbar />
      
      <main className="relative z-10 flex-1 flex flex-col items-center justify-center">
        {/* HERO SECTION */}
        <section 
          className="w-full max-w-7xl mx-auto px-6 lg:px-8 flex flex-col items-center justify-center text-center relative"
          style={{
             transform: `translateY(${scrollY * 0.15}px)`,
             opacity: Math.max(1 - scrollY / 500, 0),
          }}
        >
          {/* Subtle warm glow behind hero */}
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[600px] bg-[rgba(250,249,246,0.02)] blur-[120px] rounded-full pointer-events-none" />

          <h1 
            className="text-[clamp(2.5rem,8vw,6.5rem)] leading-[0.95] tracking-[-0.04em] text-[var(--text)] mb-4 relative z-10 w-full"
            style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
          >
            <span className="block whitespace-normal md:whitespace-nowrap">The middleman layer</span>
            <span className="block whitespace-normal md:whitespace-nowrap">for AI inference.</span>
          </h1>
          
          <p 
            className="text-[18px] md:text-[22px] text-[#e2e2e2] opacity-90 md:text-[var(--text-muted)] md:opacity-100 max-w-2xl mb-8 relative z-10"
            style={{ fontFamily: "var(--font-matter)", lineHeight: 1.5 }}
          >
            Use AI subscriptions as upstream providers for inference.
          </p>

          <div className="relative z-10 mb-16 px-2">
            <button
              onClick={handleCopy}
              className="flex items-center justify-between gap-4 md:gap-6 px-4 py-3 md:px-5 md:py-3.5 bg-[rgba(14,14,13,0.8)] border border-[rgba(226,226,226,0.12)] rounded-xl transition-colors duration-200 backdrop-blur-md hover:bg-[rgba(255,255,255,0.05)] cursor-pointer w-full max-w-[400px] overflow-hidden"
            >
              <div className="flex items-center gap-[1.5ch]">
                <span className="text-[var(--text-muted)] opacity-50 select-none font-mono text-[14px]">$</span>
                <code className="text-[14px] font-mono text-[var(--text)] flex items-center gap-[1ch]">
                  <span className="animate-pkg-width inline-flex h-[1.4em] overflow-hidden text-left">
                    <span className="animate-pkg-scroll flex flex-col w-full">
                      <span className="flex h-[1.4em] items-center w-full">npm</span>
                      <span className="flex h-[1.4em] items-center w-full">bun</span>
                      <span className="flex h-[1.4em] items-center w-full">pnpm</span>
                      <span className="flex h-[1.4em] items-center w-full">npm</span>
                    </span>
                  </span>
                  <span className="whitespace-pre">i  -g  @dhruv2mars/gunmetal</span>
                </code>
              </div>
              <div className="flex items-center justify-end ml-2 min-w-[85px]">
                {copied ? (
                  <div className="flex items-center gap-2 animate-in text-[var(--text)]">
                    <span className="text-[11px] font-mono uppercase tracking-wider">Copied</span>
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M5 13l4 4L19 7" />
                    </svg>
                  </div>
                ) : (
                  <svg className="w-4 h-4 text-[var(--text-muted)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                )}
              </div>
            </button>
          </div>
        </section>


      </main>

      <Footer />
    </div>
  );
}
