import type { Metadata } from "next";
import {
  CodeBlock,
  PageFrame,
  PageIntro,
  Panel,
  TextLink,
  localApiUrl,
  localAppUrl,
} from "@/components/ui/MarketingPrimitives";

export const metadata: Metadata = {
  title: "Documentation",
  description: "Guides and API references for Gunmetal.",
};

const docsSections = [
  {
    id: "install",
    step: "01",
    title: "Install",
    body: "Install the CLI globally. The npm wrapper installs the native runtime into your local Gunmetal home.",
    command: "npm i -g @dhruv2mars/gunmetal",
    points: ["Public entrypoint for the CLI.", "Use `gunmetal setup` after install."],
  },
  {
    id: "setup",
    step: "02",
    title: "Set up one provider",
    body: "Run the guided setup. It connects one provider, checks auth, syncs models, creates one local key, and prints the first test command.",
    command: "gunmetal setup",
    points: ["Browser-session providers open auth.", "API-key providers store upstream keys locally."],
  },
  {
    id: "run",
    step: "03",
    title: "Run the local API",
    body: "Use the Web UI when you want the browser operator surface. Use start when you only need the OpenAI-compatible API.",
    command: `gunmetal web\n# ${localAppUrl}\n# ${localApiUrl}`,
    points: ["Web UI and API share the same daemon.", "Request history stays local."],
  },
  {
    id: "client",
    step: "04",
    title: "Point one app at Gunmetal",
    body: "Any client that accepts a custom OpenAI-compatible base URL, API key, and arbitrary model id can talk to Gunmetal.",
    command: `OPENAI_BASE_URL=${localApiUrl}\nOPENAI_API_KEY=gm_your_local_key\nMODEL=provider/model`,
    points: ["Use a Gunmetal key, not an upstream provider key.", "Models use provider prefixes."],
  },
];

const apiRows = [
  ["GET", "/v1/models", "List synced provider models visible to the key."],
  ["POST", "/v1/chat/completions", "OpenAI-compatible chat completion endpoint."],
  ["POST", "/v1/responses", "Responses-style endpoint for compatible providers."],
];

export default function DocsPage() {
  return (
    <PageFrame>
      <PageIntro
        eyebrow="Resources"
        title="Documentation"
        body="A compact path through the real Gunmetal workflow: install, setup, local API, client config, and recovery."
      />

      <section className="mt-16 grid gap-10 md:grid-cols-[180px_1fr] md:gap-12">
        <aside className="hidden md:block">
          <div className="sticky top-24 border-l border-[rgba(226,226,226,0.10)] pl-4">
            <p className="font-mono text-[11px] uppercase tracking-[0.18em] text-[var(--text-faint)]">Quick start</p>
            <nav className="mt-5 flex flex-col gap-3">
              {docsSections.map((section) => (
                <a
                  key={section.id}
                  href={`#${section.id}`}
                  className="group text-[14px] text-[var(--text-muted)] transition-colors duration-150 hover:text-[var(--text)]"
                >
                  <span className="font-mono text-[12px] text-[var(--text-faint)]">{section.step}</span>{" "}
                  {section.title}
                </a>
              ))}
            </nav>
          </div>
        </aside>

        <div className="min-w-0">
          {docsSections.map((section) => (
            <section
              key={section.id}
              id={section.id}
              className="scroll-mt-28 border-t border-[rgba(226,226,226,0.08)] py-10 first:pt-0"
            >
              <div className="flex flex-col gap-2">
                <span className="font-mono text-[12px] text-[var(--text-faint)]">Step {section.step}</span>
                <h2 className="text-[28px] font-normal leading-tight text-[var(--text)]">{section.title}</h2>
              </div>
              <p className="mt-4 max-w-[64ch] text-[15px] leading-7 text-[var(--text-muted)]">{section.body}</p>
              <div className="mt-6">
                <CodeBlock>{section.command}</CodeBlock>
              </div>
              <ul className="mt-5 space-y-2">
                {section.points.map((point) => (
                  <li
                    key={point}
                    className="relative pl-4 text-[14px] leading-6 text-[var(--text-muted)] before:absolute before:left-0 before:top-[0.7em] before:h-1 before:w-1 before:rounded-full before:bg-[var(--text-faint)]"
                  >
                    {point}
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      </section>

      <Panel className="mt-16 p-5">
        <div className="grid gap-5 md:grid-cols-[180px_1fr] md:gap-12">
          <div>
            <p className="font-mono text-[11px] uppercase tracking-[0.18em] text-[var(--text-faint)]">API</p>
            <h2 className="mt-3 text-[24px] font-normal text-[var(--text)]">Contract</h2>
          </div>
          <div className="grid gap-3">
            {apiRows.map(([method, path, body]) => (
              <article key={path} className="grid gap-2 rounded-lg border border-[rgba(226,226,226,0.08)] p-4 sm:grid-cols-[70px_1fr]">
                <span className="font-mono text-[12px] text-[var(--text-faint)]">{method}</span>
                <div>
                  <p className="font-mono text-[13px] text-[var(--text-secondary)]">{path}</p>
                  <p className="mt-2 text-[14px] leading-6 text-[var(--text-muted)]">{body}</p>
                </div>
              </article>
            ))}
            <div className="pt-2">
              <TextLink href="/changelogs">Read changelogs</TextLink>
            </div>
          </div>
        </div>
      </Panel>
    </PageFrame>
  );
}
