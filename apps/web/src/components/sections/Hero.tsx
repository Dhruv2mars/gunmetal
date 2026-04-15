"use client";

import { motion, useScroll, useTransform } from "framer-motion";
import { useRef } from "react";
import { FilmGrain } from "@/components/ui/FilmGrain";

export function Hero() {
  const containerRef = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ["start start", "end start"],
  });

  // Parallax effects for the eclipse elements
  const yText = useTransform(scrollYProgress, [0, 1], [0, 150]);
  const yEclipse = useTransform(scrollYProgress, [0, 1], [0, 80]);
  const opacityText = useTransform(scrollYProgress, [0, 0.5], [1, 0]);

  const handleCopy = () => {
    navigator.clipboard.writeText("npm i -g @dhruv2mars/gunmetal");
  };

  return (
    <section 
      ref={containerRef}
      className="relative min-h-[100vh] w-full flex flex-col items-center justify-center overflow-hidden"
      style={{ background: "var(--bg)" }}
    >
      <FilmGrain />
      
      {/* 
        ========================================================================
        THE ECLIPSE (Utterly Subtle and Refined)
        ========================================================================
      */}
      <motion.div 
        style={{ y: yEclipse }}
        className="absolute inset-0 flex items-center justify-center pointer-events-none z-0"
      >
        <div className="relative w-[1000px] h-[1000px] flex items-center justify-center">
          
          {/* Barely perceptible ambient corona */}
          <div 
            className="absolute inset-0 rounded-full opacity-[0.04]"
            style={{
              background: "radial-gradient(circle, rgba(250,249,246,1) 0%, transparent 65%)",
              filter: "blur(80px)",
            }}
          />

          {/* The Proxy Sphere */}
          <motion.div 
            initial={{ scale: 0.96, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={{ duration: 2.5, ease: [0.16, 1, 0.3, 1] }}
            className="absolute w-[700px] h-[700px] sm:w-[800px] sm:h-[800px] rounded-full z-10"
            style={{
              background: "var(--bg)",
              // A single, microscopic 1px inner rim light and ultra-soft outer glow
              boxShadow: `
                inset 0 1px 1px rgba(250,249,246, 0.02),
                inset 0 40px 80px -40px rgba(250,249,246, 0.015),
                0 -20px 120px -20px rgba(250,249,246, 0.03)
              `,
            }}
          />
        </div>
      </motion.div>

      {/* 
        ========================================================================
        THE FOREGROUND (Typography & Actions)
        ========================================================================
      */}
      <motion.div 
        style={{ y: yText, opacity: opacityText }}
        className="relative z-20 container mx-auto px-6 flex flex-col items-center text-center mt-[-8vh]"
      >
        <motion.h1 
          initial={{ opacity: 0, y: 15 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1.2, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
          className="mb-8 max-w-[1000px] text-[52px] sm:text-[64px] md:text-[88px] font-normal leading-[0.96] tracking-[-0.04em] text-[#faf9f6]"
          style={{ fontFamily: "var(--font-matter)" }}
        >
          <span className="block drop-shadow-[0_4px_24px_rgba(0,0,0,0.8)]">The middleman layer</span>
          <span className="block text-[#868584]">for AI inference.</span>
        </motion.h1>

        <motion.p
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1.2, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
          className="mx-auto mb-16 max-w-[600px] text-[18px] md:text-[22px] text-[#afaeac] leading-[1.5] tracking-[-0.01em]"
          style={{ fontFamily: "var(--font-matter)" }}
        >
          Use AI subscriptions as upstream providers for inference.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1.2, delay: 0.6, ease: [0.16, 1, 0.3, 1] }}
          className="flex justify-center w-full"
        >
          <button 
            onClick={handleCopy}
            className="group relative flex items-center justify-between gap-6 rounded-[8px] border border-[rgba(250,249,246,0.08)] bg-[rgba(250,249,246,0.01)] backdrop-blur-sm px-6 py-[16px] transition-all duration-500 hover:border-[rgba(250,249,246,0.15)] hover:bg-[rgba(250,249,246,0.03)] w-full max-w-[440px] cursor-pointer"
          >
            <div className="relative flex items-center gap-4 z-10">
              <span className="text-[#868584] font-mono text-[14px] select-none">$</span>
              <span className="text-[#faf9f6] font-mono text-[15px] tracking-tight">npm i -g @dhruv2mars/gunmetal</span>
            </div>
            
            <div className="relative flex-shrink-0 text-[#868584] group-hover:text-[#faf9f6] transition-colors h-4 w-4 z-10">
              <svg className="w-4 h-4 opacity-100 group-active:opacity-0 transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </div>
          </button>
        </motion.div>
      </motion.div>
    </section>
  );
}