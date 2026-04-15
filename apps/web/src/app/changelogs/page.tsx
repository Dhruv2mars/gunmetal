import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Changelogs",
  description: "Latest updates and improvements to Gunmetal.",
};

export default function ChangelogsPage() {
  return (
    <section className="flex-1 flex flex-col items-center justify-center w-full max-w-7xl mx-auto px-6 lg:px-8 text-center">
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
        Changelogs
      </h1>
      <p
        className="text-[18px] text-[var(--text-muted)] max-w-lg"
        style={{ fontFamily: "var(--font-matter)", lineHeight: 1.5 }}
      >
        Coming soon.
      </p>
    </section>
  );
}
