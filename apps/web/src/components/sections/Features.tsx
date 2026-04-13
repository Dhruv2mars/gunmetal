"use client";

import { useScrollReveal } from "@/hooks/use-scroll-reveal";

const features = [
  {
    title: "Local API Gateway",
    description:
      "OpenAI-compatible REST API running on localhost. No cloud dependency, no latency overhead, no vendor lock-in.",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6a3 3 0 013 3m-3 0a3 3 0 00-3 3m0-6v.008h.008v-.008h-.008zm6 0h.008v.008h-.008v-.008h-.008zm0 6a3 3 0 013 3m-3 0a3 3 0 00-3 3" />
      </svg>
    ),
  },
  {
    title: "Multi-Provider Routing",
    description:
      "Connect Codex, Copilot, OpenRouter, Zen, and OpenAI simultaneously. Route by model ID like codex/gpt-5.4.",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
      </svg>
    ),
  },
  {
    title: "Request Intelligence",
    description:
      "Full request history with token usage accounting. Know exactly what's going through your stack.",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
      </svg>
    ),
  },
  {
    title: "Zero Upstream State",
    description:
      "All data stays on your machine. No telemetry, no tracking, no external dependencies. Fully local-first.",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
      </svg>
    ),
  },
  {
    title: "Browser, CLI & TUI",
    description:
      "Manage keys, profiles, and monitor requests through your preferred interface. Full feature parity across all surfaces.",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9.17 4.415a.48.48 0 00-.707.706L9.943 7.58l-2.122 2.122a.48.48 0 00.707.707l2.122-2.122 2.122 2.122a.48.48 0 00.707-.707L10.65 7.58l2.122-2.122a.48.48 0 00-.707-.707L9.943 7.58 8.464 6.1a.48.48 0 00-.707 0L6.27 7.58l2.122 2.122a.48.48 0 00.707.707L8.464 8.703l1.793 1.793a.48.48 0 00.707 0l1.793-1.793 2.122 2.122a.48.48 0 00.707-.707L10.65 9.408l1.793-1.793a.48.48 0 000-.707L11.864 6.34l2.122-2.122a.48.48 0 00-.707-.707L11.157 5.633l-1.793-1.793a.48.48 0 00-.707 0L7.17 5.633" />
      </svg>
    ),
  },
  {
    title: "Hardware Determinism",
    description:
      "Predictable performance on your hardware. No cold starts, no rate limits from your apps hitting provider APIs directly.",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M8.25 3v1.5M4.5 8.25H3m18 0h-1.5M4.5 12H3m18 0h-1.5m-15 3.75H3m18 0h-1.5M8.25 19.5V21M12 3v1.5m0 15V21m3.75-18v1.5m0 15V21m-9-1.5h10.5a2.25 2.25 0 002.25-2.25V6.75a2.25 2.25 0 00-2.25-2.25H6.75A2.25 2.25 0 004.5 6.75v10.5a2.25 2.25 0 002.25 2.25zm.75-12h9v9h-9v-9z" />
      </svg>
    ),
  },
];

export function Features() {
  const { ref, isVisible } = useScrollReveal<HTMLDivElement>();

  return (
    <section className="py-32 lg:py-40 bg-[var(--color-surface)]">
      <div className="max-w-7xl mx-auto px-6 lg:px-8">
        {/* Section header */}
        <div className="text-center mb-24">
          <span
            className={[
              "inline-block",
              "mb-6",
              "text-label",
              "text-[var(--color-accent)]",
            ].join(" ")}
          >
            Capabilities
          </span>

          <h2
            className={[
              "text-section",
              "text-[var(--color-text)]",
              "mb-6",
            ].join(" ")}
          >
            Built for those who demand control
          </h2>

          <p
            className={[
              "text-body-lg",
              "text-[var(--color-text-secondary)]",
              "max-w-lg mx-auto",
            ].join(" ")}
          >
            Everything you need to manage AI inference at scale, without
            surrendering control to third-party platforms.
          </p>
        </div>

        {/* Features grid — 2 cols tablet, 3 cols desktop */}
        <div
          ref={ref}
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
        >
          {features.map((feature, index) => (
            <div
              key={feature.title}
              className={[
                "reveal",
                isVisible ? "is-visible" : "",
              ].join(" ")}
              style={{ transitionDelay: `${index * 75}ms` }}
            >
              {/* Card — 8px radius, surface bg, border */}
              <div
                className={[
                  "h-full p-8",
                  "bg-[var(--color-bg)]",
                  "border border-[var(--color-border)]",
                  "rounded-[var(--radius-card)]",
                  "transition-all duration-200",
                  "hover:shadow-[var(--shadow-whisper)]",
                  "hover:-translate-y-0.5",
                ].join(" ")}
              >
                {/* Icon container — 40px, accent bg on hover */}
                <div
                  className={[
                    "w-10 h-10",
                    "mb-5",
                    "flex items-center justify-center",
                    "bg-[var(--color-surface)]",
                    "border border-[var(--color-border)]",
                    "rounded-[var(--radius-button)]",
                    "text-[var(--color-accent)]",
                    "transition-all duration-200",
                  ].join(" ")}
                >
                  {feature.icon}
                </div>

                {/* Title */}
                <h3
                  className={[
                    "text-subheading",
                    "text-[var(--color-text)]",
                    "mb-2",
                  ].join(" ")}
                >
                  {feature.title}
                </h3>

                {/* Description — 14px, 1.40 lh */}
                <p
                  className={[
                    "text-caption",
                    "text-[var(--color-text-secondary)]",
                    "leading-[1.40]",
                  ].join(" ")}
                >
                  {feature.description}
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
