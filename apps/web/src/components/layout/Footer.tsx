"use client";

import Link from "next/link";

const footerLinks = [
  { href: "/docs", label: "Documentation" },
  { href: "/install", label: "Install" },
  { href: "https://github.com/Dhruv2mars/gunmetal", label: "GitHub" },
  { href: "https://github.com/Dhruv2mars/gunmetal/issues", label: "Issues" },
  { href: "/changelog", label: "Changelog" },
];

export function Footer() {
  return (
    <footer className="border-t border-[var(--border)]">
      <div className="max-w-7xl mx-auto px-6 lg:px-8 py-16">
        <div className="flex flex-col lg:flex-row items-center justify-between gap-12">
          {/* Brand */}
          <div className="flex flex-col items-center lg:items-start gap-4">
            <Link href="/" className="flex items-center gap-2.5">
              <div
                className={[
                  "w-8 h-8",
                  "bg-[var(--text)]",
                  "rounded-[var(--radius-sm)]",
                  "flex items-center justify-center",
                  "font-mono font-bold text-xs",
                  "text-[var(--bg)]",
                ].join(" ")}
              >
                GM
              </div>
              <span className="text-sm font-medium text-[var(--text)]">
                Gunmetal
              </span>
            </Link>
            <p
              className={[
                "text-xs",
                "text-[var(--text-tertiary)]",
                "text-center lg:text-left",
              ].join(" ")}
            >
              Local AI inference gateway
            </p>
          </div>

          {/* Links - SpaceX minimal style, centered */}
          <div className="flex flex-wrap items-center justify-center gap-6 lg:gap-8">
            {footerLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className={[
                  "text-xs",
                  "text-[var(--text-tertiary)]",
                  "hover:text-[var(--text-secondary)]",
                  "transition-colors duration-200",
                ].join(" ")}
              >
                {link.label}
              </Link>
            ))}
          </div>

          {/* Copyright */}
          <p
            className={[
              "text-xs",
              "text-[var(--text-tertiary)]",
            ].join(" ")}
          >
            &copy; {new Date().getFullYear()}
          </p>
        </div>
      </div>
    </footer>
  );
}