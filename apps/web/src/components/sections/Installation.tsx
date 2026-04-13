"use client";

import Link from "next/link";
import { useScrollReveal } from "@/hooks/use-scroll-reveal";
import { CodeBlock } from "@/components/ui/CodeBlock";
import { Button } from "@/components/ui/Button";

const installCode = `npm install -g @dhruv2mars/gunmetal

# Verify installation
gunmetal --version

# Start the daemon
gunmetal start

# Configure providers
gunmetal setup`;

export function Installation() {
  const { ref, isVisible } = useScrollReveal<HTMLDivElement>();

  return (
    <section className="py-32 lg:py-40">
      <div className="max-w-7xl mx-auto px-6 lg:px-8">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 lg:gap-24 items-center">
          {/* Left: Text */}
          <div
            ref={ref}
            className={[
              "reveal",
              isVisible ? "is-visible" : "",
            ].join(" ")}
          >
            <span
              className={[
                "inline-block mb-6",
                "text-label",
                "text-[var(--color-accent)]",
              ].join(" ")}
            >
              Get Started
            </span>

            <h2
              className={[
                "text-section",
                "text-[var(--color-text)]",
                "mb-6",
              ].join(" ")}
            >
              Up and running
              <br />
              in minutes.
            </h2>

            <p
              className={[
                "text-body-lg",
                "text-[var(--color-text-secondary)]",
                "mb-10",
                "leading-[1.40]",
              ].join(" ")}
            >
              Install via npm, configure your providers, and start building.
              No cloud setup, no account required.
            </p>

            <Link href="/install">
              <Button variant="dark" size="lg" isPill>
                Full Installation Guide
              </Button>
            </Link>
          </div>

          {/* Right: Code block */}
          <div
            className={[
              "reveal",
              isVisible ? "is-visible" : "",
            ].join(" ")}
            style={{ transitionDelay: "150ms" }}
          >
            <CodeBlock code={installCode} language="bash" />
          </div>
        </div>
      </div>
    </section>
  );
}
