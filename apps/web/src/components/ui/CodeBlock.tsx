"use client";

import { type HTMLAttributes, useState } from "react";

interface CodeBlockProps extends HTMLAttributes<HTMLDivElement> {
  code: string;
  language?: string;
  filename?: string;
  showLineNumbers?: boolean;
}

export function CodeBlock({
  code,
  language = "bash",
  filename,
  showLineNumbers = false,
  className = "",
  ...props
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      /* silent fail */
    }
  };

  const lines = code.split("\n");

  return (
    <div
      className={[
        "relative",
        "bg-[var(--color-surface)]",
        "border border-[var(--color-border)]",
        "shadow-[var(--shadow-whisper)]",
        "overflow-hidden",
        /* 8px radius per Expo card spec */
        "rounded-[var(--radius-card)]",
        className,
      ].join(" ")}
      {...props}
    >
      {/* Header bar */}
      <div
        className={[
          "flex items-center justify-between",
          "px-4 py-2.5",
          "border-b border-[var(--color-border)]",
          "bg-[var(--color-surface-elevated)]",
        ].join(" ")}
      >
        <div className="flex items-center gap-3">
          {/* Traffic light dots */}
          <div className="flex gap-1.5">
            <span
              className="w-2.5 h-2.5 rounded-full"
              style={{ backgroundColor: "var(--color-text-tertiary)", opacity: 0.4 }}
            />
            <span
              className="w-2.5 h-2.5 rounded-full"
              style={{ backgroundColor: "var(--color-text-tertiary)", opacity: 0.4 }}
            />
            <span
              className="w-2.5 h-2.5 rounded-full"
              style={{ backgroundColor: "var(--color-text-tertiary)", opacity: 0.4 }}
            />
          </div>

          {/* Language / filename label */}
          <span
            className={[
              "text-code-small",
              "uppercase tracking-wider",
              "text-[var(--color-text-tertiary)]",
            ].join(" ")}
          >
            {filename || language}
          </span>
        </div>

        {/* Copy button */}
        <button
          onClick={handleCopy}
          className={[
            "px-2.5 py-1",
            "text-code-small",
            "text-[var(--color-text-secondary)]",
            "hover:text-[var(--color-text)]",
            "transition-colors duration-150",
            "focus:outline-none",
            "focus-visible:ring-1 focus-visible:ring-[var(--color-accent)]",
          ].join(" ")}
        >
          {copied ? "Copied" : "Copy"}
        </button>
      </div>

      {/* Code content */}
      <div className="p-4 overflow-x-auto">
        <pre
          className={[
            "text-code",
            /* JetBrains Mono — explicit */
            "font-[var(--font-mono)]",
            "leading-[1.40]",
            "text-[var(--color-text)]",
          ].join(" ")}
        >
          {lines.map((line, i) => (
            <div key={i} className="flex">
              {showLineNumbers && (
                <span
                  className={[
                    "w-8",
                    "pr-4",
                    "text-right",
                    "select-none",
                    "flex-shrink-0",
                    "text-[var(--color-text-tertiary)]",
                  ].join(" ")}
                >
                  {i + 1}
                </span>
              )}
              <code
                className={
                  showLineNumbers
                    ? "pl-4 border-l border-[var(--color-border)]"
                    : ""
                }
              >
                {line}
              </code>
            </div>
          ))}
        </pre>
      </div>
    </div>
  );
}
