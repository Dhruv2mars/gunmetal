import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Start here",
  description: "Quick path to your first request through Gunmetal.",
};

export default function StartHerePage() {
  return (
    <section className="flex-1 flex flex-col items-center justify-center w-full max-w-7xl mx-auto px-6 lg:px-8 text-center">
      <p
        className="text-[13px] uppercase tracking-[0.2em] text-[var(--text-muted)] mb-4"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
      >
        Start here
      </p>

      <h1
        className="text-[clamp(2rem,5vw,4rem)] leading-[1.05] tracking-[-0.03em] text-[var(--text)] mb-4"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
      >
        Start here
      </h1>

      <p className="text-[18px] text-[var(--text-muted)] max-w-xl mb-8" style={{ fontFamily: "var(--font-matter)" }}>
        Point your client at Gunmetal’s OpenAI-compatible API and verify connectivity.
      </p>

      <div className="w-full max-w-[760px] rounded-xl border border-[rgba(226,226,226,0.10)] bg-[rgba(14,14,13,0.55)] backdrop-blur-md p-5 text-left">
        <pre className="font-mono text-[13px] leading-relaxed text-[var(--text)] whitespace-pre-wrap">
          {`# list models
curl http://127.0.0.1:4684/v1/models

# chat completions
curl http://127.0.0.1:4684/v1/chat/completions`}
        </pre>
      </div>
    </section>
  );
}

