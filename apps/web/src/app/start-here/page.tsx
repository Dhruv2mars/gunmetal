import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Start here",
  description: "Quick path to your first request through Gunmetal.",
};

export default function StartHerePage() {
  return (
    <section className="mx-auto w-full max-w-7xl px-6 pb-24 pt-32 lg:px-8">
      <div className="grid gap-10 lg:grid-cols-[0.75fr_1.25fr]">
        <div className="border-t border-[var(--border)] pt-6">
          <p className="text-[12px] font-medium uppercase tracking-[0.26em] text-[var(--accent)]">
            Start here
          </p>
          <h1 className="mt-5 max-w-[11ch] text-[clamp(2.8rem,6vw,5.4rem)] font-semibold leading-[0.92] tracking-[-0.045em]">
            First request path.
          </h1>
          <p className="mt-6 max-w-[52ch] text-[17px] leading-8 text-[var(--text-secondary)]">
            Use this when Gunmetal is installed and setup has created at least one key and one synced model.
          </p>
        </div>
        <div className="gunmetal-panel rounded-[22px] p-5">
          <pre className="overflow-x-auto whitespace-pre-wrap rounded-[16px] bg-[rgba(6,7,8,0.32)] p-5 font-mono text-[13px] leading-6 text-[var(--text-secondary)]">{`# 1. keep the local API running
gunmetal web

# 2. verify models with a Gunmetal key
curl http://127.0.0.1:4684/v1/models \\
  -H 'Authorization: Bearer gm_your_local_key'

# 3. send chat/completions
curl http://127.0.0.1:4684/v1/chat/completions \\
  -H 'Authorization: Bearer gm_your_local_key' \\
  -H 'Content-Type: application/json' \\
  -d '{"model":"provider/model","messages":[{"role":"user","content":"say ok"}]}'`}</pre>
          <div className="mt-6 grid gap-4 md:grid-cols-[1.15fr_0.85fr]">
            <article className="border-t border-[var(--border)] pt-4">
              <h2 className="text-[20px] font-semibold">Expected shape</h2>
              <p className="mt-2 text-[14px] leading-6 text-[var(--text-muted)]">
                Base URL is local. API key starts with `gm_`. Model includes provider prefix.
              </p>
            </article>
            <article className="border-t border-[var(--border)] pt-4">
              <h2 className="text-[20px] font-semibold">When blocked</h2>
              <p className="mt-2 text-[14px] leading-6 text-[var(--text-muted)]">
                Run `gunmetal doctor`, then use the exact next command it prints.
              </p>
            </article>
          </div>
        </div>
      </div>
    </section>
  );
}
