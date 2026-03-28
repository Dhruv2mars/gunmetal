import type { Metadata } from "next";

import { SiteShell } from "@/components/site-shell";

import { PerfLabClient } from "./perf-lab-client";

export const metadata: Metadata = {
  title: "Perf Lab",
};

export default function PerfLabPage() {
  return (
    <SiteShell
      eyebrow="Perf Lab"
      title="Measure the real browser latency of upstream calls against Gunmetal."
      lede="This local-only harness sends one prompt through the exact model and config you choose, then records the time from Enter to rendered UI response. It supports openai-compatible paths plus direct local Codex."
    >
      <section className="section-grid">
        <div className="section-head">
          <p className="section-tag">How to use it</p>
          <h2>Paste a key, pick a model path, and benchmark the UI loop.</h2>
          <p>
            The OpenRouter and Zen cases use the same openai-compatible API
            shape on both sides. Codex uses a dedicated direct local path for
            the upstream case and Gunmetal&apos;s HTTP surface for the comparison
            case.
          </p>
        </div>
        <article className="warning-band">
          <p className="section-tag">Guardrail</p>
          <h2>Localhost only.</h2>
          <p>
            This page is meant to run locally while you benchmark providers.
            The matching route rejects hosted traffic, so the hosted site cannot
            become a relay for real secrets.
          </p>
        </article>
      </section>

      <PerfLabClient />
    </SiteShell>
  );
}
