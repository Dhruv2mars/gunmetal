import type { Metadata } from "next";
import { PackageManagerCommandBox } from "@/components/ui/PackageManagerCommandBox";

export const metadata: Metadata = {
  title: "Install",
  description: "Install Gunmetal from npm and get started quickly.",
};

export default function InstallPage() {
  return (
    <section className="mx-auto grid w-full max-w-7xl gap-10 px-6 pb-24 pt-32 lg:grid-cols-[0.8fr_1.2fr] lg:px-8">
      <div>
        <p className="mb-4 text-[12px] font-medium uppercase tracking-[0.26em] text-[var(--accent)]">
          Install
        </p>
        <h1 className="max-w-[10ch] text-[clamp(2.8rem,6vw,5.4rem)] font-semibold leading-[0.92] tracking-[-0.045em]">
          Put Gunmetal on this machine.
        </h1>
        <p className="mt-6 max-w-[54ch] text-[18px] leading-8 text-[var(--text-secondary)]">
          Install the CLI globally, run setup once, then keep the local API available for tools that support an OpenAI-compatible base URL.
        </p>
        <div className="mt-8">
          <PackageManagerCommandBox packageName="@dhruv2mars/gunmetal" />
        </div>
      </div>

      <div className="gunmetal-panel rounded-[22px] p-5">
        <pre className="overflow-x-auto whitespace-pre-wrap rounded-[16px] bg-[rgba(6,7,8,0.32)] p-5 font-mono text-[13px] leading-6 text-[var(--text-secondary)]">{`npm i -g @dhruv2mars/gunmetal
gunmetal setup
gunmetal web

# app config
OPENAI_BASE_URL=http://127.0.0.1:4684/v1
OPENAI_API_KEY=gm_your_local_key
MODEL=provider/model`}</pre>
        <div className="mt-6 grid gap-4 md:grid-cols-[1fr_1.35fr]">
          {[
            ["1", "Connect provider", "Browser-session providers open auth. API-key providers store upstream keys locally."],
            ["2", "Sync models", "Gunmetal records provider/model ids so every request stays explicit."],
            ["3", "Create key", "Use one Gunmetal key in clients. Do not paste upstream provider keys into apps."],
            ["4", "Send request", "Use `gunmetal chat`, the Web UI playground, or your own OpenAI-compatible client."],
          ].map(([step, title, body]) => (
            <article key={step} className="border-t border-[var(--border)] pt-4">
              <span className="font-mono text-[12px] text-[var(--accent)]">{step}</span>
              <h2 className="mt-2 text-[20px] font-semibold">{title}</h2>
              <p className="mt-2 text-[14px] leading-6 text-[var(--text-muted)]">{body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
