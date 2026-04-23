import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Documentation",
  description: "Guides and API references for Gunmetal.",
};

const sections = [
  {
    title: "Core contract",
    body: "Gunmetal issues local keys. Requests must go to Gunmetal at http://127.0.0.1:4684/v1; upstream providers never see a Gunmetal-issued key.",
  },
  {
    title: "Model ids",
    body: "Use provider-prefixed model ids such as codex/gpt-5.4, openai/gpt-5.1, or openrouter/openai/gpt-5.1.",
  },
  {
    title: "Compatibility",
    body: "Apps work when they allow a custom OpenAI-compatible base URL, custom API key, and arbitrary model string.",
  },
];

export default function DocsPage() {
  return (
    <section className="flex-1 flex flex-col items-center justify-center w-full max-w-7xl mx-auto px-6 lg:px-8 text-center py-28">
      <p
        className="text-[13px] uppercase tracking-[0.2em] text-[var(--text-muted)] mb-4"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
      >
        Resources
      </p>
      <h1
        className="text-[clamp(2rem,5vw,4rem)] leading-[1.05] tracking-[-0.03em] text-[var(--text)] mb-4"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
      >
        Documentation
      </h1>
      <p
        className="text-[18px] text-[var(--text-muted)] max-w-xl mb-8"
        style={{ fontFamily: "var(--font-matter)", lineHeight: 1.5 }}
      >
        Local-first setup, OpenAI-compatible requests, provider-prefixed models, and local request history.
      </p>

      <div className="grid w-full max-w-4xl gap-3 text-left md:grid-cols-3">
        {sections.map((section) => (
          <article
            key={section.title}
            className="rounded-xl border border-[rgba(226,226,226,0.10)] bg-[rgba(14,14,13,0.55)] p-5"
          >
            <h2 className="mb-3 text-[18px] text-[var(--text)]">{section.title}</h2>
            <p className="text-[14px] leading-relaxed text-[var(--text-muted)]">{section.body}</p>
          </article>
        ))}
      </div>
    </section>
  );
}
