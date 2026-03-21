import Link from "next/link";
import type { ReactNode } from "react";

import { primaryNav } from "@/lib/site-content";

type SiteShellProps = {
  eyebrow: string;
  title: string;
  lede: string;
  children: ReactNode;
};

export function SiteShell({ eyebrow, title, lede, children }: SiteShellProps) {
  return (
    <div className="site-frame">
      <header className="site-header">
        <Link className="brand-mark" href="/">
          <span className="brand-chip">GM</span>
          <span className="brand-copy">
            <strong>Gunmetal</strong>
            <span>Local-first AI switchboard</span>
          </span>
        </Link>
        <nav className="site-nav" aria-label="Primary">
          {primaryNav.map((item) => (
            <Link key={item.href} href={item.href}>
              {item.label}
            </Link>
          ))}
          <a
            className="nav-cta"
            href="https://github.com/Dhruv2mars/gunmetal"
            target="_blank"
            rel="noreferrer"
          >
            GitHub
          </a>
        </nav>
      </header>

      <main className="site-main">
        <section className="hero-shell">
          <p className="eyebrow">{eyebrow}</p>
          <h1>{title}</h1>
          <p className="lede">{lede}</p>
        </section>
        {children}
      </main>

      <footer className="site-footer">
        <span>One local endpoint. Explicit providers. No hosted relay.</span>
        <span>Same monorepo. Same product. Same binary.</span>
      </footer>
    </div>
  );
}
