import type { Metadata } from "next";

import { SiteShell } from "@/components/site-shell";
import { changelogEntries } from "@/lib/site-content";

export const metadata: Metadata = {
  title: "Changelog",
};

export default function ChangelogPage() {
  return (
    <SiteShell
      eyebrow="Changelog"
      title="Everything here is meant to ship, not sit as a placeholder."
      lede="Gunmetal is being built as a full product: daemon, CLI, TUI, adapters, local API, and site under one roof."
    >
      <section className="timeline">
        {changelogEntries.map((entry) => (
          <article key={`${entry.date}-${entry.title}`} className="timeline-entry">
            <p className="timeline-date">{entry.date}</p>
            <h2>{entry.title}</h2>
            <p>{entry.body}</p>
          </article>
        ))}
      </section>
    </SiteShell>
  );
}
