"use client";

import Link from "next/link";
import { useEffect, useState, useRef } from "react";
import { Button } from "@/components/ui/Button";

const headlineWords = ["The", "middleman", "layer", "for", "AI", "inference."];

export function Hero() {
  const [isLoaded, setIsLoaded] = useState(false);
  const [terminalText, setTerminalText] = useState("");
  const [showTerminal, setShowTerminal] = useState(false);
  const terminalRef = useRef<HTMLDivElement>(null);
  
  const fullCommand = "npm install -g @dhruv2mars/gunmetal";

  useEffect(() => {
    const timer = setTimeout(() => setIsLoaded(true), 100);
    return () => clearTimeout(timer);
  }, []);

  useEffect(() => {
    if (!isLoaded) return;
    
    const showDelay = setTimeout(() => {
      setShowTerminal(true);
    }, 1200);

    return () => clearTimeout(showDelay);
  }, [isLoaded]);

  useEffect(() => {
    if (!showTerminal) return;
    
    let charIndex = 0;
    const typeInterval = setInterval(() => {
      if (charIndex <= fullCommand.length) {
        setTerminalText(fullCommand.slice(0, charIndex));
        charIndex++;
      } else {
        clearInterval(typeInterval);
      }
    }, 25);

    return () => clearInterval(typeInterval);
  }, [showTerminal]);

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollLeft = terminalRef.current.scrollWidth;
    }
  }, [terminalText]);

  return (
    <section
      className={[
        "relative",
        "min-h-screen",
        "flex items-center justify-center",
        "pt-16",
        "overflow-hidden",
      ].join(" ")}
    >
      {/* Blueprint grid — subtle parallax background */}
      <div
        className={[
          "absolute inset-0",
          "grid-pattern",
          "opacity-100",
          "pointer-events-none",
        ].join(" ")}
        aria-hidden="true"
      />

      {/* Radial glow — bottom center, very subtle */}
      <div
        className={[
          "absolute bottom-0 left-1/2 -translate-x-1/2",
          "w-full max-w-3xl h-96",
          "radial-glow",
          "pointer-events-none",
        ].join(" ")}
        aria-hidden="true"
      />

      {/* Content */}
      <div
        className={[
          "relative z-10",
          "max-w-4xl mx-auto",
          "px-6 lg:px-8",
          "text-center",
        ].join(" ")}
      >
        {/* Label — uppercase, tracking wide */}
        <div
          className={[
            "mb-8",
            "opacity-0",
            isLoaded ? "animate-fade-in-up" : "",
          ].join(" ")}
          style={{ animationDelay: "200ms" }}
        >
          <span
            className={[
              "text-label",
              "text-[var(--text-secondary)]",
            ].join(" ")}
          >
            Local AI Inference Gateway
          </span>
        </div>

        {/* Headline — word by word reveal */}
        <h1
          className={[
            "text-display",
            "text-[var(--text)]",
            "mb-8",
            "leading-[1.05]",
          ].join(" ")}
        >
          {headlineWords.map((word, index) => (
            <span
              key={`${word}-${index}`}
              className={[
                "inline-block",
                "mr-[0.25em]",
                "opacity-0",
                isLoaded ? "animate-word-reveal" : "",
              ].join(" ")}
              style={{
                animationDelay: `${400 + index * 60}ms`,
              }}
            >
              {word}
            </span>
          ))}
        </h1>

        {/* Subheading */}
        <p
          className={[
            "text-body-lg",
            "text-[var(--text-secondary)]",
            "max-w-xl mx-auto",
            "mb-12",
            "opacity-0",
            isLoaded ? "animate-fade-in" : "",
          ].join(" ")}
          style={{ animationDelay: "1000ms" }}
        >
          Turn your AI subscriptions into one local API for inference.
        </p>

        {/* Terminal block — typewriter effect */}
        <div
          className={[
            "mb-12",
            "inline-flex items-center",
            "max-w-full overflow-hidden",
            "opacity-0",
            isLoaded ? "animate-fade-in" : "",
          ].join(" ")}
          style={{ animationDelay: "1200ms" }}
        >
          <div
            ref={terminalRef}
            className={[
              "flex items-center gap-3",
              "px-5 py-3",
              "bg-[var(--surface)]",
              "border border-[var(--border)]",
              "rounded-[var(--radius-card)]",
              "overflow-x-auto",
            ].join(" ")}
          >
            <span
              className={[
                "text-code-small",
                "text-[var(--text-tertiary)]",
                "flex-shrink-0",
              ].join(" ")}
            >
              $
            </span>
            <code
              className={[
                "text-code",
                "text-[var(--text)]",
                "whitespace-nowrap",
              ].join(" ")}
            >
              {terminalText}
              <span
                className={[
                  "inline-block w-2 h-4",
                  "bg-[var(--text-secondary)]",
                  "ml-0.5",
                  "animate-pulse",
                ].join(" ")}
                aria-hidden="true"
              />
            </code>
          </div>
        </div>

        {/* CTAs — spark effect on hover */}
        <div
          className={[
            "flex flex-col sm:flex-row",
            "items-center justify-center",
            "gap-4",
            "opacity-0",
            isLoaded ? "animate-fade-in-scale" : "",
          ].join(" ")}
          style={{ animationDelay: "1600ms" }}
        >
          <Link href="/install" className="w-full sm:w-auto">
            <Button
              variant="primary"
              size="lg"
              className="w-full sm:w-auto spark-hover"
            >
              Install Gunmetal
            </Button>
          </Link>
          <Link href="/docs" className="w-full sm:w-auto">
            <Button
              variant="ghost"
              size="lg"
              className="w-full sm:w-auto"
            >
              Documentation
            </Button>
          </Link>
        </div>
      </div>

      {/* Scroll indicator — subtle bounce, delayed */}
      <div
        className={[
          "absolute bottom-8 left-1/2 -translate-x-1/2",
          "opacity-0",
          isLoaded ? "animate-fade-in" : "",
          "animate-bounce-subtle",
        ].join(" ")}
        style={{ animationDelay: "2000ms" }}
        aria-hidden="true"
      >
        <svg
          className="w-5 h-5 text-[var(--text-tertiary)]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </div>
    </section>
  );
}