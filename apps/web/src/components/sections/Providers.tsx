"use client";

import { useScrollReveal } from "@/hooks/use-scroll-reveal";

const providers = [
  {
    name: "Codex",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5" />
      </svg>
    ),
  },
  {
    name: "GitHub Copilot",
    icon: (
      <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
        <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.64.7 1.028 1.595 1.028 2.688 0 3.848-2.807 5.624-5.479 5.921.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
      </svg>
    ),
  },
  {
    name: "OpenRouter",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418" />
      </svg>
    ),
  },
  {
    name: "Zen",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z" />
      </svg>
    ),
  },
  {
    name: "OpenAI",
    icon: (
      <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
        <path d="M22.282 9.821a5.985 5.985 0 0 0-.516-4.91 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a5.985 5.985 0 0 0-3.998 2.9 6.046 6.046 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.997-2.9 6.056 6.056 0 0 0-.747-7.073zM13.26 22.43v-5.28a5.44 5.44 0 0 0-.321-2.19 5.507 5.507 0 0 0 2.94-2.207 5.413 5.413 0 0 0 1.525 1.525 5.507 5.507 0 0 0-2.19 2.94h-5.28a5.503 5.503 0 0 0 3.326-1.237v5.28zm1.21-9.126a5.43 5.43 0 0 0-2.19.321v5.28a5.44 5.44 0 0 0 2.19.321v-5.28a5.44 5.44 0 0 1-.321-2.19h5.28a5.507 5.507 0 0 1 2.207 2.94 5.413 5.413 0 0 1-1.525-1.525 5.507 5.507 0 0 1-2.94 2.207v-5.28zM21.54 9.822a5.413 5.413 0 0 1-1.525 1.525 5.507 5.507 0 0 1-2.94-2.207 5.44 5.44 0 0 1-.321-2.19h5.28a5.507 5.507 0 0 1 2.19.321v5.28a5.44 5.44 0 0 0-.322-2.19v-2.55zm-9.937 7.427v-2.19a5.413 5.413 0 0 1 1.525-1.525 5.507 5.507 0 0 0 2.94 2.207h-2.19v5.28a5.44 5.44 0 0 0 .322 2.19h2.55a5.413 5.413 0 0 1-1.525-1.525 5.507 5.507 0 0 0-2.94-2.207h5.28a5.44 5.44 0 0 0 .321-2.19zm3.905-1.985l1.785 1.073 1.072 1.783-1.072 1.784-1.784 1.073-1.073-1.784-.645-1.073-.646-1.073.646-1.073zm5.522-.646l1.073 1.784 1.784 1.073-1.073 1.784-1.784 1.072-1.073-1.784-.645-1.073-.646-1.073.646-1.073z" />
      </svg>
    ),
  },
];

export function Providers() {
  const { ref, isVisible } = useScrollReveal<HTMLDivElement>();

  return (
    <section className="py-24 lg:py-32 border-t border-[var(--border)]">
      <div className="max-w-6xl mx-auto px-6 lg:px-8">
        {/* Section header */}
        <div className="text-center mb-16">
          <h2
            className={[
              "text-section",
              "text-[var(--text)]",
            ].join(" ")}
          >
            Connect everything
          </h2>
        </div>

        {/* Providers row — minimal icons, no cards */}
        <div
          ref={ref}
          className={[
            "flex flex-wrap items-center justify-center gap-8 lg:gap-16",
            "reveal",
            isVisible ? "is-visible" : "",
          ].join(" ")}
        >
          {providers.map((provider) => (
            <div
              key={provider.name}
              className={[
                "flex items-center gap-2",
                "text-[var(--text-secondary)]",
                "transition-colors duration-200",
                "hover:text-[var(--text)]",
              ].join(" ")}
            >
              <span className="opacity-70">{provider.icon}</span>
              <span
                className={[
                  "text-sm font-medium",
                  "hidden sm:block",
                ].join(" ")}
              >
                {provider.name}
              </span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}