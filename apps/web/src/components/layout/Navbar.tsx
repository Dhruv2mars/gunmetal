"use client";

import Link from "next/link";
import { useState, useEffect } from "react";

const navItems = [
  {
    label: "Products",
    items: [
      {
        label: "Gunmetal Suite",
        desc: "The all-in-one platform for your needs.",
        href: "/products/suite",
        icon: (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
        ),
      },
    ],
  },
  {
    label: "Developer",
    items: [
      {
        label: "Extension SDK",
        desc: "Build powerful native integrations.",
        href: "/developer/sdk",
        icon: (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
          </svg>
        ),
      },
    ],
  },
  {
    label: "Resources",
    items: [
      {
        label: "Documentation",
        desc: "Guides and API references.",
        href: "/docs",
        icon: (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
          </svg>
        ),
      },
      {
        label: "Changelogs",
        desc: "Latest updates and improvements.",
        href: "/changelogs",
        icon: (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        ),
      },
    ],
  },
  { label: "Download", href: "/download" },
];

export function Navbar() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [openAccordion, setOpenAccordion] = useState<string | null>(null);
  const [isScrolled, setIsScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 30);
    };

    window.addEventListener("scroll", handleScroll, { passive: true });
    handleScroll();

    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const toggleAccordion = (label: string) => {
    setOpenAccordion(openAccordion === label ? null : label);
  };

  return (
    <header
      className="fixed top-0 left-0 right-0 z-50 transition-all duration-300"
      style={{
        background: "rgba(14, 14, 13, 0.70)",
        backdropFilter: "blur(20px) saturate(180%)",
        WebkitBackdropFilter: "blur(20px) saturate(180%)",
        boxShadow: "0 0 0 1px rgba(226, 226, 226, 0.06)",
      }}
    >
      <nav className="w-full max-w-7xl mx-auto px-6 lg:px-8">
        <div className="flex items-center justify-between h-14">
          {/* Logo — left */}
          <Link href="/" className="flex items-center group flex-shrink-0 h-full">
            <img
              src="/logo.svg"
              alt="Gunmetal"
              className="h-[22px] w-auto flex-shrink-0 relative z-10 bg-transparent opacity-70 group-hover:opacity-100 transition-opacity duration-200"
              style={{ display: "block" }}
            />
            {/* Mask Container */}
            <div
              className={`overflow-hidden transition-all duration-700 ease-[cubic-bezier(0.16,1,0.3,1)] ml-1.5 flex items-center ${
                isScrolled ? "max-w-0 opacity-0" : "max-w-[150px] opacity-100"
              }`}
            >
              <span
                className={`block text-[20px] leading-none tracking-tight text-[var(--text-muted)] group-hover:text-[var(--text)] whitespace-nowrap transition-all duration-700 ease-[cubic-bezier(0.16,1,0.3,1)] relative -top-[0.5px] ${
                  isScrolled ? "-translate-x-full" : "translate-x-0"
                }`}
                style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
              >
                Gunmetal
              </span>
            </div>
          </Link>

          {/* Nav links + GitHub — right side, tight grouping */}
          <div className="hidden lg:flex items-center gap-2 h-full">
            {navItems.map((item) => (
              <div key={item.label} className="relative group h-full flex items-center justify-center min-w-[110px]">
                {item.items ? (
                  <button
                    className="flex items-center gap-1.5 text-[14px] text-[var(--text-muted)] group-hover:text-[var(--text)] transition-colors duration-200 h-full"
                    style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
                  >
                    {item.label}
                    <svg
                      className="w-3.5 h-3.5 text-[var(--text-muted)] group-hover:text-[var(--text)] transition-transform duration-300 group-hover:rotate-180"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                      strokeWidth={3}
                    >
                      <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
                    </svg>
                  </button>
                ) : (
                  <Link
                    href={item.href!}
                    className="flex items-center text-[14px] text-[var(--text-muted)] group-hover:text-[var(--text)] transition-colors duration-200 h-full"
                    style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
                  >
                    {item.label}
                  </Link>
                )}

                {/* Dropdown Menu */}
                {item.items && (
                  <div className="absolute top-[calc(100%-0.15rem)] left-1/2 -translate-x-1/2 pt-1 opacity-0 pointer-events-none group-hover:opacity-100 group-hover:pointer-events-auto transition-all duration-300 ease-out transform translate-y-2 group-hover:translate-y-0 z-50">
                    <div
                      className="rounded-lg p-1.5 w-max min-w-[220px] grid gap-1.5"
                      style={{
                        background: "rgba(14, 14, 13, 0.85)",
                        backdropFilter: "blur(24px) saturate(200%)",
                        boxShadow: "0 10px 40px -10px rgba(0,0,0,0.7), 0 0 0 1px rgba(226, 226, 226, 0.1)",
                      }}
                    >
                      {item.items.map((subItem) => (
                        <Link
                          key={subItem.href}
                          href={subItem.href}
                          className="group/item flex items-center gap-3 p-2.5 text-[var(--text-muted)] hover:text-[var(--text)] hover:bg-[var(--frosted)] rounded-lg transition-all duration-200"
                        >
                          <div className="shrink-0 text-[var(--text-muted)] group-hover/item:text-[var(--text)] transition-colors duration-200">
                            {subItem.icon}
                          </div>
                          <div className="flex flex-col gap-0.5">
                            <span
                              className="text-[14px] font-medium leading-none text-[var(--text-muted)] group-hover/item:text-[var(--text)] transition-colors duration-200"
                              style={{ fontFamily: "var(--font-matter)" }}
                            >
                              {subItem.label}
                            </span>
                            <span className="text-[13px] leading-snug text-[var(--text-muted)]">
                              {subItem.desc}
                            </span>
                          </div>
                        </Link>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ))}

            {/* GitHub */}
            <a
              href="https://github.com/Dhruv2mars/gunmetal"
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center justify-center w-8 h-8 rounded-lg text-[var(--text-muted)] hover:text-[var(--text)] transition-colors duration-200 flex-shrink-0"
              aria-label="GitHub"
            >
              <svg className="w-[20px] h-[20px]" fill="currentColor" viewBox="0 0 24 24">
                <path
                  fillRule="evenodd"
                  clipRule="evenodd"
                  d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                />
              </svg>
            </a>
          </div>

          {/* Mobile Right Section */}
          <div className="lg:hidden flex items-center gap-3 h-full">
            <a
              href="https://github.com/Dhruv2mars/gunmetal"
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center justify-center w-8 h-8 rounded-lg text-[var(--text-muted)] hover:text-[var(--text)] transition-colors duration-200 flex-shrink-0"
              aria-label="GitHub"
            >
              <svg className="w-[20px] h-[20px]" fill="currentColor" viewBox="0 0 24 24">
                <path
                  fillRule="evenodd"
                  clipRule="evenodd"
                  d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                />
              </svg>
            </a>
            
            <button
              onClick={() => setMobileOpen(!mobileOpen)}
              className="flex items-center justify-center w-8 h-8 rounded-lg text-[var(--text-muted)] hover:text-[var(--text)] transition-colors duration-200"
              aria-label="Menu"
              aria-expanded={mobileOpen}
            >
              {mobileOpen ? (
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              ) : (
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M4 6h16M4 12h16M4 18h16" />
                </svg>
              )}
            </button>
          </div>
        </div>

        {/* Mobile menu */}
        <div
          className={`lg:hidden overflow-hidden transition-all duration-300 ease-in-out ${
            mobileOpen ? "max-h-[800px] opacity-100" : "max-h-0 opacity-0"
          }`}
        >
          <div className="bg-[rgba(14,14,13,0.98)] backdrop-blur-3xl shadow-xl">
            <div className="px-4 py-6 flex flex-col gap-1 border-t border-[rgba(226,226,226,0.08)] min-h-screen">
            {navItems.map((item) => (
              <div key={item.label} className="flex flex-col">
                {item.items ? (
                  <>
                    <button
                      onClick={() => toggleAccordion(item.label)}
                      className="flex items-center justify-between w-full px-4 py-3 text-[14px] text-[var(--text)] hover:bg-[var(--frosted)] rounded-lg transition-colors duration-200"
                      style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
                    >
                      {item.label}
                      <svg
                        className={`w-4 h-4 text-[var(--text-muted)] transition-transform duration-300 ${
                          openAccordion === item.label ? "rotate-180" : ""
                        }`}
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        strokeWidth={3}
                      >
                        <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
                      </svg>
                    </button>
                    
                    {/* Mobile Accordion Content */}
                    <div
                      className={`overflow-hidden transition-all duration-300 ease-in-out ${
                        openAccordion === item.label ? "max-h-[400px] opacity-100 mt-1 mb-3" : "max-h-0 opacity-0 mt-0 mb-0"
                      }`}
                    >
                      <div className="flex flex-col pl-4 border-l border-[rgba(226,226,226,0.15)] ml-6 gap-1">
                        {item.items.map((subItem) => (
                          <Link
                            key={subItem.href}
                            href={subItem.href}
                            onClick={() => setMobileOpen(false)}
                            className="py-2.5 px-4 text-[13px] text-[var(--text-muted)] hover:text-[var(--text)] transition-colors duration-200"
                            style={{ fontFamily: "var(--font-matter)", fontWeight: 400 }}
                          >
                            {subItem.label}
                          </Link>
                        ))}
                      </div>
                    </div>
                  </>
                ) : (
                  <Link
                    href={item.href!}
                    onClick={() => setMobileOpen(false)}
                    className="px-4 py-3 text-[14px] text-[var(--text)] hover:bg-[var(--frosted)] rounded-lg transition-colors duration-200"
                    style={{ fontFamily: "var(--font-matter)", fontWeight: 500 }}
                  >
                    {item.label}
                  </Link>
                )}
              </div>
            ))}
          </div>
          </div>
        </div>
      </nav>
    </header>
  );
}
