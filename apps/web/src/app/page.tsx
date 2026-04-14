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
    <div className="min-h-screen bg-[var(--bg)] text-[var(--text)] selection:bg-[#faf9f6] selection:text-[#1a1a19] relative overflow-hidden">
      <Navbar />
      
      <main className="relative z-10 pt-32 pb-8">
        {/* HERO SECTION */}
        <section 
          className="w-full max-w-7xl mx-auto px-6 lg:px-8 py-32 md:py-48 flex flex-col items-center justify-center text-center relative min-h-[60vh]"
          style={{
             transform: `translateY(${scrollY * 0.15}px)`,
             opacity: Math.max(1 - scrollY / 500, 0),
          }}
        >
          {/* Subtle warm glow behind hero */}
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[600px] bg-[rgba(250,249,246,0.02)] blur-[120px] rounded-full pointer-events-none" />

          <h1 
            className="text-[clamp(3.5rem,10vw,6rem)] leading-[1.0] tracking-[-2.4px] text-[var(--text)] mb-8 relative z-10 max-w-4xl"
            style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
          >
            The middleman layer<br />for AI inference.
          </h1>
          
          <p 
            className="text-[18px] md:text-[20px] text-[var(--text-muted)] max-w-2xl mb-12 relative z-10"
            style={{ fontFamily: "var(--font-matter)", lineHeight: 1.5 }}
          >
            Use AI subscriptions as upstream providers for inference.
          </p>

          <div className="relative z-10 mb-16">
            <button
              onClick={handleCopy}
              className="flex items-center justify-between gap-6 px-4 py-3 md:px-5 md:py-3.5 bg-[rgba(14,14,13,0.6)] border border-[rgba(226,226,226,0.06)] rounded-xl transition-colors duration-200 backdrop-blur-md hover:bg-[rgba(255,255,255,0.03)] cursor-pointer"
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

        {/* FEATURES GRID */}
        <section className="w-full max-w-7xl mx-auto px-6 lg:px-8 py-24">
          <div className="mb-16">
             <h2 className="text-[12px] tracking-[2.4px] uppercase text-[var(--text-muted)] mb-4" style={{ fontFamily: "var(--font-matter)" }}>
               Core Philosophy
             </h2>
             <h3 
               className="text-[clamp(2rem,4vw,3rem)] leading-[1.1] tracking-[-0.96px] text-[var(--text)] max-w-2xl"
               style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
             >
               Everything runs through a single, secure tunnel.
             </h3>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
             {/* Card 1 */}
             <div 
               className="p-8 rounded-[12px] group transition-colors duration-300 hover:bg-[rgba(255,255,255,0.03)]"
               style={{
                 background: "rgba(255, 255, 255, 0.015)",
                 boxShadow: "0 0 0 1px rgba(226, 226, 226, 0.06)",
               }}
             >
               <div className="w-10 h-10 mb-6 rounded-lg bg-[rgba(255,255,255,0.04)] flex items-center justify-center text-[var(--text-muted)] group-hover:text-[var(--text)] transition-colors duration-300">
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                  </svg>
               </div>
               <h4 className="text-[20px] text-[var(--text)] mb-3" style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}>Local Keys Only</h4>
               <p className="text-[16px] text-[var(--text-secondary)] leading-relaxed" style={{ fontFamily: "var(--font-matter)" }}>
                 Your API keys never leave your machine. Gunmetal securely proxies all inference requests through a local daemon.
               </p>
             </div>

             {/* Card 2 */}
             <div 
               className="p-8 rounded-[12px] group transition-colors duration-300 hover:bg-[rgba(255,255,255,0.03)]"
               style={{
                 background: "rgba(255, 255, 255, 0.015)",
                 boxShadow: "0 0 0 1px rgba(226, 226, 226, 0.06)",
               }}
             >
               <div className="w-10 h-10 mb-6 rounded-lg bg-[rgba(255,255,255,0.04)] flex items-center justify-center text-[var(--text-muted)] group-hover:text-[var(--text)] transition-colors duration-300">
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 002-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                  </svg>
               </div>
               <h4 className="text-[20px] text-[var(--text)] mb-3" style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}>Universal Protocol</h4>
               <p className="text-[16px] text-[var(--text-secondary)] leading-relaxed" style={{ fontFamily: "var(--font-matter)" }}>
                 Talk to Claude, GPT, or local models using a single unified API format. Swap models without changing code.
               </p>
             </div>

             {/* Card 3 */}
             <div 
               className="p-8 rounded-[12px] group transition-colors duration-300 hover:bg-[rgba(255,255,255,0.03)]"
               style={{
                 background: "rgba(255, 255, 255, 0.015)",
                 boxShadow: "0 0 0 1px rgba(226, 226, 226, 0.06)",
               }}
             >
               <div className="w-10 h-10 mb-6 rounded-lg bg-[rgba(255,255,255,0.04)] flex items-center justify-center text-[var(--text-muted)] group-hover:text-[var(--text)] transition-colors duration-300">
                  <svg className="w-5 h-5" fill="none" viewBox="0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
               </div>
               <h4 className="text-[20px] text-[var(--text)] mb-3" style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}>Native Performance</h4>
               <p className="text-[16px] text-[var(--text-secondary)] leading-relaxed" style={{ fontFamily: "var(--font-matter)" }}>
                 Built in Rust. The daemon runs silently in the background with zero overhead, ready exactly when you need it.
               </p>
             </div>
          </div>
        </section>

        {/* BOTTOM CTA */}
        <section className="w-full max-w-7xl mx-auto px-6 lg:px-8 py-32 text-center flex flex-col items-center">
           <h2 
             className="text-[clamp(2.5rem,6vw,4rem)] leading-[1.0] tracking-[-1.5px] text-[var(--text)] mb-8"
             style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
           >
             Ready to build?
           </h2>
           <a 
             href="/docs"
             className="group inline-flex items-center gap-2 text-[16px] text-[var(--text-muted)] hover:text-[var(--text)] transition-colors duration-300"
             style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
           >
             Read the installation guide
             <svg 
               className="w-4 h-4 text-[var(--text-muted)] group-hover:text-[var(--text)] transition-all duration-300 group-hover:translate-x-1" 
               fill="none" 
               viewBox="0 0 24 24" 
               stroke="currentColor" 
               strokeWidth={2}
             >
               <path strokeLinecap="round" strokeLinejoin="round" d="M17 8l4 4m0 0l-4 4m4-4H3" />
             </svg>
           </a>
        </section>
      </main>

      <Footer />
    </div>
  );
}
