"use client";

import { motion } from "framer-motion";
import { SpotlightCard } from "@/components/ui/SpotlightCard";

const features = [
  {
    title: "Connect Your Providers",
    description: "Link your existing accounts like OpenAI, OpenRouter, Anthropic, or local LLMs through unified integrations.",
    icon: (
      <svg className="h-6 w-6 text-[#faf9f6]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
      </svg>
    ),
  },
  {
    title: "Mint Gunmetal Keys",
    description: "Generate local, scoped API keys that grant precise access to specific models and providers.",
    icon: (
      <svg className="h-6 w-6 text-[#faf9f6]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
      </svg>
    ),
  },
  {
    title: "Use Anywhere",
    description: "Point your apps to the local endpoint, inject your Gunmetal key, and access all your models instantly.",
    icon: (
      <svg className="h-6 w-6 text-[#faf9f6]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
      </svg>
    ),
  }
];

export function Bento() {
  return (
    <section className="relative w-full pb-32 pt-16" style={{ background: "var(--bg)" }}>
      <div className="container mx-auto px-6 max-w-6xl">
        <div className="mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-100px" }}
            transition={{ duration: 0.6 }}
            className="text-3xl md:text-[40px] font-normal leading-[1.1] tracking-[-0.4px] text-[#faf9f6]"
            style={{ fontFamily: "var(--font-matter)" }}
          >
            One local proxy.<br />
            <span className="text-[#868584]">Zero external dependencies.</span>
          </motion.h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <motion.div
              key={index}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: "-50px" }}
              transition={{ duration: 0.6, delay: index * 0.15 }}
              className="h-full"
            >
              <SpotlightCard className="h-full p-8 bg-[rgba(255,255,255,0.01)] backdrop-blur-sm group">
                <div className="mb-6 inline-flex h-12 w-12 items-center justify-center rounded-lg bg-[rgba(255,255,255,0.03)] border border-[rgba(226,226,226,0.05)] transition-colors group-hover:bg-[rgba(255,255,255,0.06)]">
                  {feature.icon}
                </div>
                <h3 
                  className="mb-3 text-[22px] font-medium leading-[1.14] text-[#faf9f6]"
                  style={{ fontFamily: "var(--font-matter)" }}
                >
                  {feature.title}
                </h3>
                <p 
                  className="text-[16px] leading-[1.4] text-[#afaeac]"
                  style={{ fontFamily: "var(--font-matter)" }}
                >
                  {feature.description}
                </p>
              </SpotlightCard>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}