import Link from "next/link";
import type { ReactNode } from "react";

export const repoUrl = "https://github.com/Dhruv2mars/gunmetal";
export const releasesUrl = `${repoUrl}/releases`;
export const packageName = "@dhruv2mars/gunmetal";
export const installCommand = `npm i -g ${packageName}`;
export const localApiUrl = "http://127.0.0.1:4684/v1";
export const localAppUrl = "http://127.0.0.1:4684/app";

export function PageFrame({ children }: { children: ReactNode }) {
  return (
    <main className="relative mx-auto w-full max-w-[1000px] px-6 pb-28 pt-28 md:px-8 md:pt-32">
      {children}
    </main>
  );
}

export function PageIntro({
  eyebrow,
  title,
  body,
}: {
  eyebrow: string;
  title: string;
  body: string;
}) {
  return (
    <section className="border-t border-[rgba(226,226,226,0.10)] pt-6">
      <p className="font-mono text-[11px] font-medium uppercase tracking-[0.22em] text-[var(--text-muted)]">
        {eyebrow}
      </p>
      <h1
        className="mt-5 max-w-[13ch] text-[clamp(2.7rem,7vw,5.6rem)] font-normal leading-[0.96] tracking-[0] text-[var(--text)]"
        style={{ fontFamily: "var(--font-matter)" }}
      >
        {title}
      </h1>
      <p className="mt-6 max-w-[58ch] text-[17px] leading-8 text-[var(--text-muted)] md:text-[18px]">
        {body}
      </p>
    </section>
  );
}

export function Panel({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <section
      className={[
        "rounded-xl border border-[rgba(226,226,226,0.10)] bg-[rgba(14,14,13,0.62)]",
        "shadow-[0_24px_80px_-48px_rgba(0,0,0,0.9)] backdrop-blur-md",
        className,
      ].join(" ")}
    >
      {children}
    </section>
  );
}

export function CodeBlock({ children }: { children: string }) {
  return (
    <pre className="overflow-x-auto rounded-lg border border-[rgba(226,226,226,0.08)] bg-[rgba(6,7,8,0.36)] p-4 font-mono text-[13px] leading-6 text-[var(--text-secondary)]">
      {children}
    </pre>
  );
}

export function TextLink({
  href,
  children,
}: {
  href: string;
  children: ReactNode;
}) {
  const external = href.startsWith("http");
  const className =
    "inline-flex items-center gap-1 text-[14px] font-medium text-[var(--text)] transition-colors duration-150 hover:text-[var(--text-muted)]";

  if (external) {
    return (
      <a href={href} target="_blank" rel="noreferrer" className={className}>
        {children}
        <span aria-hidden="true">↗</span>
      </a>
    );
  }

  return (
    <Link href={href} className={className}>
      {children}
      <span aria-hidden="true">→</span>
    </Link>
  );
}

export function NumberedRow({
  number,
  title,
  body,
  children,
}: {
  number: string;
  title: string;
  body: string;
  children?: ReactNode;
}) {
  return (
    <article className="grid gap-5 border-t border-[rgba(226,226,226,0.08)] py-8 md:grid-cols-[150px_1fr] md:gap-10">
      <div>
        <span className="font-mono text-[12px] text-[var(--text-faint)]">{number}</span>
      </div>
      <div className="min-w-0">
        <h2 className="text-[24px] font-normal leading-tight text-[var(--text)]">{title}</h2>
        <p className="mt-3 max-w-[62ch] text-[15px] leading-7 text-[var(--text-muted)]">{body}</p>
        {children ? <div className="mt-5">{children}</div> : null}
      </div>
    </article>
  );
}
