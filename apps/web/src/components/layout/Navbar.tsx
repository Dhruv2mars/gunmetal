"use client";

import Image from "next/image";
import Link from "next/link";
import { useEffect, useState } from "react";

const navItems = [
  { href: "/install", label: "Install" },
  { href: "/start-here", label: "Start" },
  { href: "/webui", label: "Web UI" },
  { href: "/docs", label: "Docs" },
];

function GitHubLink({ className = "" }: { className?: string }) {
  return (
    <a
      href="https://github.com/Dhruv2mars/gunmetal"
      target="_blank"
      rel="noreferrer"
      className={`inline-flex items-center justify-center rounded-lg text-[var(--text-muted)] transition-colors duration-200 hover:text-[var(--text)] ${className}`}
      aria-label="GitHub"
    >
      <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path
          fillRule="evenodd"
          clipRule="evenodd"
          d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
        />
      </svg>
    </a>
  );
}

export function Navbar() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [isScrolled, setIsScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => setIsScrolled(window.scrollY > 30);
    window.addEventListener("scroll", handleScroll, { passive: true });
    handleScroll();
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <header
      className="pointer-events-auto fixed left-0 right-0 top-0 z-[100] transition-all duration-300"
      style={{
        background: "rgba(17, 17, 15, 0.78)",
        backdropFilter: "blur(20px) saturate(180%)",
        WebkitBackdropFilter: "blur(20px) saturate(180%)",
        boxShadow: "0 0 0 1px rgba(226, 226, 226, 0.06)",
      }}
    >
      <nav className="mx-auto w-full max-w-7xl px-6 lg:px-8" aria-label="Primary">
        <div className="flex h-14 items-center justify-between">
          <Link href="/" className="flex h-full flex-shrink-0 items-center group" aria-label="Gunmetal Home">
            <Image
              src="/logo.svg"
              alt=""
              width={22}
              height={22}
              aria-hidden="true"
              className="relative z-10 h-[22px] w-auto flex-shrink-0 bg-transparent opacity-70 transition-opacity duration-200 group-hover:opacity-100"
            />
            <div
            className={`ml-1.5 flex items-center overflow-hidden transition-all duration-700 ease-[cubic-bezier(0.16,1,0.3,1)] ${
                isScrolled ? "max-w-0 opacity-0" : "max-w-[150px] opacity-100"
              }`}
            >
              <span
                className={`relative -top-[0.5px] block whitespace-nowrap text-[20px] leading-none tracking-tight text-[var(--text-muted)] transition-all duration-700 ease-[cubic-bezier(0.16,1,0.3,1)] group-hover:text-[var(--text)] ${
                  isScrolled ? "-translate-x-full" : "translate-x-0"
                }`}
                style={{ fontFamily: "var(--font-sans)", fontWeight: 600 }}
              >
                Gunmetal
              </span>
            </div>
          </Link>

          <div className="hidden h-full items-center gap-2 lg:flex">
            {navItems.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className="flex h-full items-center px-2 text-[14px] text-[var(--text-muted)] transition-colors duration-200 hover:text-[var(--text)]"
                style={{ fontFamily: "var(--font-sans)", fontWeight: 500 }}
              >
                {item.label}
              </Link>
            ))}
            <GitHubLink className="h-8 w-8" />
          </div>

          <div className="flex h-full items-center gap-3 lg:hidden">
            <GitHubLink className="h-8 w-8" />
            <button
              onClick={() => setMobileOpen((open) => !open)}
              className="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--text-muted)] transition-colors duration-200 hover:text-[var(--text)]"
              aria-label="Menu"
              aria-expanded={mobileOpen}
              type="button"
            >
              {mobileOpen ? (
                <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              ) : (
                <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M4 6h16M4 12h16M4 18h16" />
                </svg>
              )}
            </button>
          </div>
        </div>

        <div
          className={`overflow-hidden transition-all duration-300 ease-in-out lg:hidden ${
            mobileOpen ? "max-h-[420px] opacity-100" : "max-h-0 opacity-0"
          }`}
        >
          <div className="min-h-[100dvh] border-t border-[rgba(226,226,226,0.08)] bg-[rgba(17,17,15,0.98)] px-4 py-6 shadow-xl backdrop-blur-3xl">
            <div className="flex flex-col gap-1">
              {navItems.map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  onClick={() => setMobileOpen(false)}
                  className="rounded-lg px-4 py-3 text-[14px] text-[var(--text)] transition-colors duration-200 hover:bg-[var(--frosted)]"
                  style={{ fontFamily: "var(--font-sans)", fontWeight: 500 }}
                >
                  {item.label}
                </Link>
              ))}
            </div>
          </div>
        </div>
      </nav>
    </header>
  );
}
