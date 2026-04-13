"use client";

import Link from "next/link";
import { useState, useEffect } from "react";

export function Navbar() {
  const [isScrolled, setIsScrolled] = useState(false);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 50);
    };

    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <header
      className={[
        "fixed top-0 left-0 right-0 z-50",
        "h-16 flex items-center",
        "transition-all duration-300",
        isScrolled
          ? "bg-[var(--bg)]/90 backdrop-blur-xl border-b border-[var(--border)]"
          : "bg-transparent",
      ].join(" ")}
    >
      <nav className="max-w-7xl mx-auto px-6 lg:px-8 w-full">
        <div className="flex items-center justify-between h-full">
          {/* Logo */}
          <Link href="/" className="flex items-center gap-2">
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
            <span
              className={[
                "hidden sm:block",
                "text-sm font-medium tracking-tight",
                "text-[var(--text)]",
              ].join(" ")}
            >
              Gunmetal
            </span>
          </Link>

          {/* Desktop Nav - simple, centered */}
          <div className="hidden lg:flex items-center gap-8">
            <Link
              href="/docs"
              className="text-sm text-[var(--text-secondary)] hover:text-[var(--text)] transition-colors duration-200"
            >
              Docs
            </Link>
            <Link
              href="/install"
              className="text-sm text-[var(--text-secondary)] hover:text-[var(--text)] transition-colors duration-200"
            >
              Install
            </Link>
            <Link
              href="/start-here"
              className="text-sm text-[var(--text-secondary)] hover:text-[var(--text)] transition-colors duration-200"
            >
              Start Here
            </Link>
          </div>

          {/* Right side - GitHub + mobile menu */}
          <div className="flex items-center gap-4">
            <a
              href="https://github.com/Dhruv2mars/gunmetal"
              target="_blank"
              rel="noreferrer"
              className={[
                "hidden sm:flex",
                "items-center gap-2",
                "text-xs font-medium",
                "text-[var(--text-secondary)]",
                "transition-colors duration-200",
                "hover:text-[var(--text)]",
              ].join(" ")}
              aria-label="View on GitHub"
            >
              <svg
                className="w-4 h-4"
                fill="currentColor"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <path
                  fillRule="evenodd"
                  d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                  clipRule="evenodd"
                />
              </svg>
            </a>

            {/* Mobile menu button */}
            <button
              onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
              className={[
                "lg:hidden",
                "p-2",
                "text-[var(--text-secondary)]",
                "hover:text-[var(--text)]",
                "transition-colors duration-150",
              ].join(" ")}
              aria-label="Toggle menu"
              aria-expanded={isMobileMenuOpen}
            >
              {isMobileMenuOpen ? (
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M6 18L18 6M6 6l12 12" />
                </svg>
              ) : (
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 6h16M4 12h16M4 18h16" />
                </svg>
              )}
            </button>
          </div>
        </div>

        {/* Mobile menu */}
        {isMobileMenuOpen && (
          <div className="lg:hidden absolute top-16 left-0 right-0 bg-[var(--bg)]/95 backdrop-blur-xl border-b border-[var(--border)]">
            <div className="px-6 py-6 flex flex-col gap-4">
              <Link
                href="/docs"
                onClick={() => setIsMobileMenuOpen(false)}
                className="block py-2 text-[var(--text-secondary)] hover:text-[var(--text)] transition-colors duration-200"
              >
                Docs
              </Link>
              <Link
                href="/install"
                onClick={() => setIsMobileMenuOpen(false)}
                className="block py-2 text-[var(--text-secondary)] hover:text-[var(--text)] transition-colors duration-200"
              >
                Install
              </Link>
              <Link
                href="/start-here"
                onClick={() => setIsMobileMenuOpen(false)}
                className="block py-2 text-[var(--text-secondary)] hover:text-[var(--text)] transition-colors duration-200"
              >
                Start Here
              </Link>
              <a
                href="https://github.com/Dhruv2mars/gunmetal"
                target="_blank"
                rel="noreferrer"
                className="block py-2 text-[var(--text-secondary)] hover:text-[var(--text)] transition-colors duration-200"
              >
                GitHub
              </a>
            </div>
          </div>
        )}
      </nav>
    </header>
  );
}