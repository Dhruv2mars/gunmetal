"use client";

import { useEffect, useMemo, useRef, useState } from "react";

type PackageManager = "npm" | "bun" | "pnpm";

const loopManagers: PackageManager[] = ["npm", "bun", "pnpm", "npm"];

function usePrefersReducedMotion() {
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => setReduced(media.matches);
    update();
    media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  }, []);

  return reduced;
}

export function PackageManagerCommandBox({
  packageName,
  tail = "i -g",
  cycleMs = 2500,
  className = "",
}: {
  packageName: string;
  tail?: string;
  cycleMs?: number;
  className?: string;
}) {
  const prefersReducedMotion = usePrefersReducedMotion();
  const [managerIndex, setManagerIndex] = useState(0); // 0..3 (last item is duplicate for seamless loop)
  const [copied, setCopied] = useState(false);
  const copiedTimeoutRef = useRef<number | null>(null);
  const resetTimeoutRef = useRef<number | null>(null);
  const [disableTransition, setDisableTransition] = useState(false);

  const manager = (loopManagers[managerIndex] ?? "npm") as PackageManager;

  const copyCommand = useMemo(() => `${manager} ${tail} ${packageName}`, [manager, tail, packageName]);
  const displayTail = useMemo(() => `${tail.replace(/\s+/g, " ")}  ${packageName}`, [tail, packageName]);

  useEffect(() => {
    if (prefersReducedMotion) return;
    const id = window.setInterval(() => {
      setManagerIndex((i) => {
        const next = i + 1;
        return next > loopManagers.length - 1 ? 0 : next;
      });
    }, cycleMs);
    return () => window.clearInterval(id);
  }, [cycleMs, prefersReducedMotion]);

  useEffect(() => {
    if (prefersReducedMotion) return;
    // When we reach the duplicate last row, snap back to 0 without animation.
    if (managerIndex !== loopManagers.length - 1) return;
    if (resetTimeoutRef.current) window.clearTimeout(resetTimeoutRef.current);
    resetTimeoutRef.current = window.setTimeout(() => {
      setDisableTransition(true);
      setManagerIndex(0);
      // Re-enable transitions on next tick.
      window.setTimeout(() => setDisableTransition(false), 30);
    }, 740); // just after 700ms transition completes
  }, [managerIndex, prefersReducedMotion]);

  useEffect(() => {
    return () => {
      if (copiedTimeoutRef.current) window.clearTimeout(copiedTimeoutRef.current);
      if (resetTimeoutRef.current) window.clearTimeout(resetTimeoutRef.current);
    };
  }, []);

  const handleCopy = async () => {
    let didCopy = false;

    try {
      await navigator.clipboard.writeText(copyCommand);
      didCopy = true;
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = copyCommand;
      textarea.setAttribute("readonly", "");
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      textarea.style.pointerEvents = "none";
      document.body.appendChild(textarea);
      textarea.select();
      didCopy = document.execCommand("copy");
      textarea.remove();
    }

    if (!didCopy) return;

    setCopied(true);
    if (copiedTimeoutRef.current) window.clearTimeout(copiedTimeoutRef.current);
    copiedTimeoutRef.current = window.setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={`relative z-10 px-2 ${className}`}>
      <button
        onClick={handleCopy}
        type="button"
        className="flex items-center justify-between gap-4 md:gap-6 px-4 py-3 md:px-5 md:py-3.5 bg-[rgba(14,14,13,0.8)] border border-[rgba(226,226,226,0.12)] rounded-xl transition-colors duration-200 backdrop-blur-md hover:bg-[rgba(255,255,255,0.05)] cursor-pointer w-full max-w-[500px] overflow-hidden"
        aria-label={`Copy install command: ${copyCommand}`}
      >
        <div className="flex items-center gap-[1.5ch] min-w-0">
          <span className="text-[var(--text-muted)] opacity-50 select-none font-mono text-[14px]" aria-hidden="true">
            $
          </span>

          <code className="text-[14px] font-mono text-[var(--text)] flex items-center gap-[1ch] min-w-0">
            <span
              className="inline-flex h-[1.4em] w-[4ch] overflow-hidden text-left"
              aria-hidden="true"
            >
              <span
                className="flex flex-col w-full"
                style={{
                  transform: `translateY(${-managerIndex * 1.4}em)`,
                  transition:
                    prefersReducedMotion || disableTransition
                      ? "none"
                      : "transform 700ms cubic-bezier(0.87, 0, 0.13, 1)",
                }}
              >
                <span className="flex h-[1.4em] items-center w-full">npm</span>
                <span className="flex h-[1.4em] items-center w-full">bun</span>
                <span className="flex h-[1.4em] items-center w-full">pnpm</span>
                <span className="flex h-[1.4em] items-center w-full">npm</span>
              </span>
            </span>

            <span className="whitespace-pre">{displayTail}</span>
            <span className="sr-only">{copyCommand}</span>
          </code>
        </div>

        <div className="flex items-center justify-end ml-2 min-w-[92px]">
          <span className="sr-only" aria-live="polite">
            {copied ? "Copied" : ""}
          </span>

          {copied ? (
            <div className="flex items-center gap-2 animate-in text-[var(--text)]">
              <span className="text-[11px] font-mono uppercase tracking-wider">Copied</span>
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M5 13l4 4L19 7" />
              </svg>
            </div>
          ) : (
            <svg
              className="w-4 h-4 text-[var(--text-muted)]"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
              />
            </svg>
          )}
        </div>
      </button>
    </div>
  );
}
