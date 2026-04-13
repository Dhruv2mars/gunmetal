"use client";

import { useState, useRef, useEffect, type ReactNode } from "react";

interface DropdownItem {
  label: string;
  description?: string;
  href?: string;
  onClick?: () => void;
}

interface DropdownProps {
  trigger: ReactNode;
  items: DropdownItem[];
  align?: "left" | "right";
}

export function Dropdown({ trigger, items, align = "left" }: DropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen]);

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Trigger */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        onMouseEnter={() => setIsOpen(true)}
        className={[
          "flex items-center gap-1",
          "text-caption font-medium",
          "text-[var(--color-text-secondary)]",
          "hover:text-[var(--color-text)]",
          "transition-colors duration-150",
          "focus:outline-none",
          "focus-visible:ring-2 focus-visible:ring-[var(--color-accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--color-bg)]",
        ].join(" ")}
        aria-expanded={isOpen}
        aria-haspopup="true"
      >
        {trigger}
        {/* Chevron */}
        <svg
          className={[
            "w-3 h-3",
            "transition-transform duration-200",
            isOpen ? "rotate-180" : "",
          ].join(" ")}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth={2}
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Menu */}
      {isOpen && (
        <div
          className={[
            "absolute top-full mt-2",
            "w-64",
            "bg-[var(--color-surface)]",
            "border border-[var(--color-border)]",
            "shadow-[var(--shadow-elevated)]",
            "rounded-[var(--radius-card)]",
            "overflow-hidden",
            "z-50",
            "animate-in",
            align === "right" ? "right-0" : "left-0",
          ].join(" ")}
          style={{ animationDuration: "150ms" }}
        >
          <div className="py-1">
            {items.map((item, index) => (
              <div key={index}>
                {item.href ? (
                  <a
                    href={item.href}
                    className={[
                      "block",
                      "px-4 py-3",
                      "hover:bg-[var(--color-surface-elevated)]",
                      "transition-colors duration-150",
                    ].join(" ")}
                    onClick={() => setIsOpen(false)}
                  >
                    <div className="text-body font-medium text-[var(--color-text)]">
                      {item.label}
                    </div>
                    {item.description && (
                      <div className="text-caption text-[var(--color-text-secondary)] mt-0.5">
                        {item.description}
                      </div>
                    )}
                  </a>
                ) : (
                  <button
                    onClick={() => {
                      item.onClick?.();
                      setIsOpen(false);
                    }}
                    className={[
                      "w-full text-left",
                      "px-4 py-3",
                      "hover:bg-[var(--color-surface-elevated)]",
                      "transition-colors duration-150",
                    ].join(" ")}
                  >
                    <div className="text-body font-medium text-[var(--color-text)]">
                      {item.label}
                    </div>
                    {item.description && (
                      <div className="text-caption text-[var(--color-text-secondary)] mt-0.5">
                        {item.description}
                      </div>
                    )}
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
