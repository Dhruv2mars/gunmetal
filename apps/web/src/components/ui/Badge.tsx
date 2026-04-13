"use client";

import { type HTMLAttributes, forwardRef } from "react";

type BadgeVariant = "default" | "accent" | "success";

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

const variantStyles: Record<BadgeVariant, string> = {
  default:
    "bg-[var(--color-surface)] text-[var(--color-text-secondary)] border border-[var(--color-border)]",
  accent:
    "bg-[var(--color-accent)] text-white border border-transparent",
  success:
    "bg-[var(--color-success)] text-white border border-transparent",
};

export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(
  ({ className = "", variant = "default", children, ...props }, ref) => {
    return (
      <span
        ref={ref}
        className={[
          "inline-flex items-center",
          "px-2.5 py-1",
          "text-code-small",
          "rounded-full",
          variantStyles[variant],
          className,
        ].join(" ")}
        {...props}
      >
        {children}
      </span>
    );
  }
);

Badge.displayName = "Badge";
