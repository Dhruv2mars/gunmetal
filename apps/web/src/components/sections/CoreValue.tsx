"use client";

import { useScrollReveal } from "@/hooks/use-scroll-reveal";

const values = [
  {
    title: "One endpoint. Every provider.",
    description:
      "Connect Codex, Copilot, OpenRouter, Zen, and OpenAI simultaneously. Route by model ID like codex/gpt-5.4 — Gunmetal handles the rest.",
    icon: (
      <svg
        className="w-6 h-6"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        strokeWidth={1.5}
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M5.25 8.25h13.5M5.25 12h13.5m0 7.5h-13.5"
        />
      </svg>
    ),
  },
  {
    title: "Full request visibility.",
    description:
      "History, token accounting, and usage intelligence — all stored locally. Know exactly what's running through your stack.",
    icon: (
      <svg
        className="w-6 h-6"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        strokeWidth={1.5}
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z"
        />
      </svg>
    ),
  },
  {
    title: "Built on open foundations.",
    description:
      "Extension SDK for multi-provider routing, public in future. Build on the same primitives that power Gunmetal.",
    icon: (
      <svg
        className="w-6 h-6"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        strokeWidth={1.5}
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5"
        />
      </svg>
    ),
  },
];

export function CoreValue() {
  const { ref, isVisible } = useScrollReveal<HTMLDivElement>();

  return (
    <section className="py-32 lg:py-40 bg-[var(--color-surface)]">
      <div className="max-w-7xl mx-auto px-6 lg:px-8">
        {/* Section header */}
        <div className="text-center mb-20">
          <h2
            className={[
              "text-section",
              "text-[var(--color-text)]",
            ].join(" ")}
          >
            Why Gunmetal
          </h2>
        </div>

        {/* Values grid */}
        <div
          ref={ref}
          className="grid grid-cols-1 lg:grid-cols-3 gap-8 lg:gap-12"
        >
          {values.map((value, index) => (
            <div
              key={value.title}
              className={[
                "reveal",
                isVisible ? "is-visible" : "",
              ].join(" ")}
              style={{ transitionDelay: `${index * 100}ms` }}
            >
              {/* Card */}
              <div
                className={[
                  "h-full",
                  "p-10",
                  "bg-[var(--color-bg)]",
                  "border border-[var(--color-border)]",
                  "rounded-[var(--radius-card)]",
                ].join(" ")}
              >
                {/* Icon */}
                <div
                  className={[
                    "w-12 h-12",
                    "mb-6",
                    "flex items-center justify-center",
                    "bg-[var(--color-surface)]",
                    "border border-[var(--color-border)]",
                    "rounded-[var(--radius-button)]",
                    "text-[var(--color-accent)]",
                  ].join(" ")}
                >
                  {value.icon}
                </div>

                {/* Title */}
                <h3
                  className={[
                    "text-subheading",
                    "text-[var(--color-text)]",
                    "mb-3",
                  ].join(" ")}
                >
                  {value.title}
                </h3>

                {/* Description */}
                <p
                  className={[
                    "text-body",
                    "text-[var(--color-text-secondary)]",
                    "leading-[1.40]",
                  ].join(" ")}
                >
                  {value.description}
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
