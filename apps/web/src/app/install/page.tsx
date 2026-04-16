import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Install",
  description: "Install Gunmetal from npm and get started quickly.",
};

export default function InstallPage() {
  return (
    <section className="flex-1 flex flex-col items-center justify-center w-full max-w-7xl mx-auto px-6 lg:px-8 text-center">
      <p
        className="text-[13px] uppercase tracking-[0.2em] text-[var(--text-muted)] mb-4"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
      >
        Get started
      </p>
      <h1
        className="text-[clamp(2rem,5vw,4rem)] leading-[1.05] tracking-[-0.03em] text-[var(--text)] mb-5"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
      >
        Install
      </h1>

      <div className="w-full max-w-[720px] rounded-xl border border-[rgba(226,226,226,0.10)] bg-[rgba(14,14,13,0.55)] backdrop-blur-md p-5 text-left">
        <pre className="font-mono text-[13px] leading-relaxed text-[var(--text)] whitespace-pre-wrap">
          {`npm i -g @dhruv2mars/gunmetal
gunmetal setup`}
        </pre>
      </div>
    </section>
  );
}

