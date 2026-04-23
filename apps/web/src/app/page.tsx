import { PackageManagerCommandBox } from "@/components/ui/PackageManagerCommandBox";
import Link from "next/link";

export default function HomePage() {
  return (
    <div className="relative w-full overflow-hidden pt-24">
      <div className="hairline-grid pointer-events-none absolute inset-x-0 top-0 h-[560px]" />
      <section className="relative mx-auto grid w-full max-w-7xl gap-10 px-6 pb-20 pt-14 lg:grid-cols-[1.05fr_0.95fr] lg:px-8 lg:pt-24">
        <div className="max-w-3xl">
          <p className="mb-5 text-[12px] font-medium uppercase tracking-[0.28em] text-[var(--accent)]">
            Local AI gateway
          </p>
          <h1 className="max-w-[12ch] text-[clamp(3rem,8vw,6.6rem)] font-semibold leading-[0.9] tracking-[-0.045em] text-[var(--text)]">
            One local API for your AI providers.
          </h1>
          <p className="mt-7 max-w-[58ch] text-[18px] leading-8 text-[var(--text-secondary)] md:text-[20px]">
            Gunmetal turns provider accounts into one OpenAI-compatible endpoint on your machine. Use one local key, choose provider/model ids, then inspect every request when things break.
          </p>
          <div className="mt-9 flex flex-col gap-4 sm:flex-row sm:items-center">
            <PackageManagerCommandBox packageName="@dhruv2mars/gunmetal" />
            <Link
              href="/start-here"
              className="inline-flex min-h-12 items-center justify-center rounded-lg border border-[var(--border-strong)] px-5 text-[14px] font-medium text-[var(--text)] transition duration-200 hover:-translate-y-px hover:bg-[var(--frosted)] active:translate-y-0"
            >
              Read first request path
            </Link>
          </div>
        </div>

        <aside className="gunmetal-panel rounded-[22px] p-4 lg:mt-10">
          <div className="rounded-[16px] border border-[var(--border)] bg-[rgba(17,17,15,0.78)] p-5">
            <div className="mb-5 flex items-center justify-between gap-3 border-b border-[var(--border)] pb-4">
              <div>
                <p className="text-[12px] uppercase tracking-[0.22em] text-[var(--text-muted)]">Runbook</p>
                <h2 className="mt-2 text-[22px] font-semibold tracking-[-0.02em]">Install to first request</h2>
              </div>
              <span className="rounded-full border border-[rgba(125,190,179,0.34)] bg-[rgba(125,190,179,0.1)] px-3 py-1 text-[12px] text-[var(--success)]">
                local
              </span>
            </div>
            <pre className="overflow-x-auto whitespace-pre-wrap rounded-[14px] bg-[rgba(6,7,8,0.34)] p-4 font-mono text-[13px] leading-6 text-[var(--text-secondary)]">{`npm i -g @dhruv2mars/gunmetal
gunmetal setup
gunmetal web

OPENAI_BASE_URL=http://127.0.0.1:4684/v1
OPENAI_API_KEY=gm_your_local_key
MODEL=provider/model`}</pre>
            <div className="mt-5 grid gap-3 sm:grid-cols-[1fr_1.35fr]">
              {[
                ["provider", "Codex, OpenAI, OpenRouter, custom upstreams"],
                ["model", "provider/model ids stay explicit"],
                ["key", "local Gunmetal key, upstream keys stay local"],
                ["request", "latency, tokens, status, error trail"],
              ].map(([label, body]) => (
                <div key={label} className="border-t border-[var(--border)] pt-3">
                  <p className="font-mono text-[12px] uppercase tracking-[0.16em] text-[var(--accent)]">{label}</p>
                  <p className="mt-2 text-[14px] leading-6 text-[var(--text-muted)]">{body}</p>
                </div>
              ))}
            </div>
          </div>
        </aside>
      </section>

      <section className="mx-auto grid w-full max-w-7xl gap-5 px-6 pb-24 lg:grid-cols-[0.75fr_1.25fr] lg:px-8">
        <div className="border-t border-[var(--border)] pt-6">
          <p className="text-[12px] uppercase tracking-[0.22em] text-[var(--text-muted)]">Operator surfaces</p>
          <h2 className="mt-4 max-w-[12ch] text-[42px] font-semibold leading-none tracking-[-0.04em]">
            CLI speed. Browser clarity.
          </h2>
        </div>
        <div className="grid gap-4 md:grid-cols-[1.15fr_0.85fr]">
          <article className="gunmetal-panel rounded-[18px] p-6">
            <h3 className="text-[24px] font-semibold">CLI</h3>
            <p className="mt-3 max-w-[56ch] text-[15px] leading-7 text-[var(--text-muted)]">
              Setup, start, chat, diagnose, and inspect traffic from terminal output that stays copyable.
            </p>
            <pre className="mt-5 overflow-x-auto rounded-[14px] bg-[rgba(6,7,8,0.28)] p-4 font-mono text-[13px] leading-6 text-[var(--text-secondary)]">{`gunmetal doctor
gunmetal chat --model codex/gpt-5.4
gunmetal logs summary`}</pre>
          </article>
          <article className="rounded-[18px] border border-[var(--border)] p-6">
            <h3 className="text-[24px] font-semibold">Web UI</h3>
            <p className="mt-3 text-[15px] leading-7 text-[var(--text-muted)]">
              Connect providers, sync models, mint keys, test requests, and read history at the local daemon.
            </p>
            <Link
              href="/webui"
              className="mt-5 inline-flex min-h-11 items-center rounded-lg bg-[var(--accent)] px-4 text-[14px] font-semibold text-[#171716] transition duration-200 hover:-translate-y-px active:translate-y-0"
            >
              Open Web UI guide
            </Link>
          </article>
        </div>
      </section>
    </div>
  );
}
