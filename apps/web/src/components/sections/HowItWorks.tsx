"use client";

import { useScrollReveal } from "@/hooks/use-scroll-reveal";

const steps = [
  {
    number: "01",
    title: "Mount Providers",
    description:
      "Connect Codex, Copilot, OpenRouter, Zen, or any OpenAI-compatible provider. Authenticate once, route forever.",
  },
  {
    number: "02",
    title: "Mint Access",
    description:
      "Create local API keys with scopes, limits, and expiry. Full control without upstream state.",
  },
  {
    number: "03",
    title: "Direct Apps",
    description:
      "Point applications to your local endpoint. Use provider-prefixed model IDs like codex/gpt-5.4.",
  },
];

export function HowItWorks() {
  const { ref, isVisible } = useScrollReveal<HTMLDivElement>();

  return (
    <section className="py-32 lg:py-40">
      <div className="max-w-6xl mx-auto px-6 lg:px-8">
        {/* Section header */}
        <div className="text-center mb-20">
          <span
            className={[
              "inline-block",
              "mb-6",
              "text-label",
              "text-[var(--text-secondary)]",
            ].join(" ")}
          >
            How It Works
          </span>

          <h2
            className={[
              "text-section",
              "text-[var(--text)]",
            ].join(" ")}
          >
            Three steps to full control
          </h2>
        </div>

        {/* Steps */}
        <div
          ref={ref}
          className="grid grid-cols-1 lg:grid-cols-3 gap-12 lg:gap-8"
        >
          {steps.map((step, index) => (
            <div
              key={step.number}
              className={[
                "reveal",
                isVisible ? "is-visible" : "",
              ].join(" ")}
              style={{ transitionDelay: `${index * 100}ms` }}
            >
              {/* Step number — large, muted, monospace */}
              <div
                className={[
                  "text-step",
                  "text-[var(--text-tertiary)]",
                  "mb-4",
                  "font-mono",
                ].join(" ")}
              >
                {step.number}
              </div>

              {/* Title */}
              <h3
                className={[
                  "text-subheading",
                  "text-[var(--text)]",
                  "mb-3",
                ].join(" ")}
              >
                {step.title}
              </h3>

              {/* Description */}
              <p
                className={[
                  "text-body",
                  "text-[var(--text-secondary)]",
                  "leading-[1.50]",
                ].join(" ")}
              >
                {step.description}
              </p>

              {/* Connector arrow — desktop only, between steps */}
              {index < steps.length - 1 && (
                <div
                  className={[
                    "hidden lg:block",
                    "absolute",
                    "right-0",
                    "top-8",
                    "text-[var(--border)]",
                  ].join(" ")}
                  style={{ transform: "translateX(calc(-50% - 2rem))" }}
                  aria-hidden="true"
                >
                  <svg
                    className="w-6 h-6"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={1}
                      d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"
                    />
                  </svg>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}