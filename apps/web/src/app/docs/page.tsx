import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Documentation",
  description: "Guides and API references for Gunmetal.",
};

const sections = [
  {
    title: "API contract",
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
    <section className="mx-auto w-full max-w-7xl px-6 pb-24 pt-32 lg:px-8">
      <div className="grid gap-10 lg:grid-cols-[0.7fr_1.3fr]">
        <div className="border-t border-[var(--border)] pt-6">
          <p className="text-[12px] font-medium uppercase tracking-[0.26em] text-[var(--accent)]">
            Docs
          </p>
          <h1 className="mt-5 max-w-[11ch] text-[clamp(2.8rem,6vw,5.4rem)] font-semibold leading-[0.92] tracking-[-0.045em]">
            Terms that matter.
          </h1>
          <p className="mt-6 max-w-[52ch] text-[17px] leading-8 text-[var(--text-secondary)]">
            Gunmetal stays small on purpose: provider, model, key, request. Learn those four and the product is predictable.
          </p>
        </div>
        <div className="grid gap-4 md:grid-cols-[1.1fr_0.9fr]">
        {sections.map((section) => (
          <article
            key={section.title}
            className="gunmetal-panel rounded-[18px] p-5"
          >
            <h2 className="mb-3 text-[20px] font-semibold text-[var(--text)]">{section.title}</h2>
            <p className="text-[14px] leading-relaxed text-[var(--text-muted)]">{section.body}</p>
          </article>
        ))}
          <article className="rounded-[18px] border border-[var(--border)] p-5 md:col-span-2">
            <h2 className="text-[20px] font-semibold">Recovery commands</h2>
            <pre className="mt-4 overflow-x-auto whitespace-pre-wrap rounded-[14px] bg-[rgba(6,7,8,0.32)] p-4 font-mono text-[13px] leading-6 text-[var(--text-secondary)]">{`gunmetal doctor
gunmetal status
gunmetal logs list --status error
gunmetal auth status <saved-provider>`}</pre>
          </article>
        </div>
      </div>
    </section>
  );
}
