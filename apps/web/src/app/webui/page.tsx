import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Web UI",
  description: "Run the local Web UI and connect via your browser.",
};

export default function WebUiPage() {
  return (
    <section className="mx-auto grid w-full max-w-7xl gap-10 px-6 pb-24 pt-32 lg:grid-cols-[0.8fr_1.2fr] lg:px-8">
      <div>
        <p className="mb-4 text-[12px] font-medium uppercase tracking-[0.26em] text-[var(--accent)]">
          Web UI
        </p>
        <h1 className="max-w-[11ch] text-[clamp(2.8rem,6vw,5.4rem)] font-semibold leading-[0.92] tracking-[-0.045em]">
          Browser control for the local daemon.
        </h1>
        <p className="mt-6 max-w-[52ch] text-[17px] leading-8 text-[var(--text-secondary)]">
          Run `gunmetal web`. It starts Gunmetal if needed, opens the local browser UI, and keeps the OpenAI-compatible API on the same machine.
        </p>
      </div>

      <div className="gunmetal-panel rounded-[22px] p-5">
        <pre className="overflow-x-auto whitespace-pre-wrap rounded-[16px] bg-[rgba(6,7,8,0.32)] p-5 font-mono text-[13px] leading-6 text-[var(--text-secondary)]">{`gunmetal web

Open: http://127.0.0.1:4684/app
API:  http://127.0.0.1:4684/v1`}</pre>
        <div className="mt-6 grid gap-4 md:grid-cols-[1fr_1fr]">
          {[
            ["Setup readiness", "See whether providers, models, keys, and traffic are ready."],
            ["Provider control", "Save provider connection details, auth browser-session providers, and sync models."],
            ["Playground", "Paste one Gunmetal key, choose a provider/model, and stream a test request."],
            ["Request history", "Inspect status, latency, token use, provider, key, endpoint, and errors."],
          ].map(([title, body]) => (
            <article key={title} className="border-t border-[var(--border)] pt-4">
              <h2 className="text-[20px] font-semibold">{title}</h2>
              <p className="mt-2 text-[14px] leading-6 text-[var(--text-muted)]">{body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
