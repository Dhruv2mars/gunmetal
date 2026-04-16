"use client";

import { PackageManagerCommandBox } from "@/components/ui/PackageManagerCommandBox";

export default function HomePage() {
  return (
    <section 
      className="flex-1 flex flex-col items-center justify-center w-full max-w-7xl mx-auto px-6 lg:px-8 text-center relative"
    >
      {/* Subtle warm glow behind hero */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[600px] bg-[rgba(250,249,246,0.02)] blur-[120px] rounded-full pointer-events-none" />

      <h1 
        className="text-[clamp(2.5rem,8vw,6.5rem)] leading-[0.95] tracking-[-0.04em] text-[var(--text)] mb-4 relative z-10 w-full max-w-[14ch] mx-auto"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
      >
        The middleman layer for AI inference.
      </h1>
      
      <p 
        className="text-[18px] md:text-[22px] text-[var(--text-muted)] opacity-90 max-w-2xl mb-8 relative z-10"
        style={{ fontFamily: "var(--font-matter)", lineHeight: 1.5 }}
      >
        Use AI subscriptions as upstream providers for inference.
      </p>

      <PackageManagerCommandBox packageName="@dhruv2mars/gunmetal" />
    </section>
  );
}
