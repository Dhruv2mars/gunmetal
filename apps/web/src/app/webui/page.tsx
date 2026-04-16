import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Web UI",
  description: "Run the local Web UI and connect via your browser.",
};

export default function WebUiPage() {
  return (
    <section className="flex-1 flex flex-col items-center justify-center w-full max-w-7xl mx-auto px-6 lg:px-8 text-center">
      <p
        className="text-[13px] uppercase tracking-[0.2em] text-[var(--text-muted)] mb-4"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
      >
        Web UI
      </p>

      <h1
        className="text-[clamp(2rem,5vw,4rem)] leading-[1.05] tracking-[-0.03em] text-[var(--text)] mb-4"
        style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
      >
        Web UI
      </h1>

      <p className="text-[18px] text-[var(--text-muted)] max-w-xl mb-8" style={{ fontFamily: "var(--font-matter)" }}>
        Run <span className="font-mono">gunmetal web</span> and open it in your browser at{" "}
        <span className="font-mono">http://127.0.0.1:4684/app</span>.
      </p>
    </section>
  );
}

