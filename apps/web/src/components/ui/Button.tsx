"use client";

import { forwardRef, type ButtonHTMLAttributes } from "react";

type ButtonVariant = "primary" | "ghost" | "spark" | "dark";
type ButtonSize = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  isPill?: boolean;
  isLoading?: boolean;
}

const primaryStyles = [
  "bg-[var(--cta-bg)]",
  "text-[var(--cta-text)]",
  "border border-[var(--cta-border)]",
  "rounded-[var(--radius-pill)]",
].join(" ");

const ghostStyles = [
  "bg-transparent",
  "text-[var(--text-secondary)]",
  "border border-[var(--border)]",
  "rounded-[var(--radius-pill)]",
].join(" ");

const sparkStyles = [
  "bg-[var(--cta-bg)]",
  "text-[var(--cta-text)]",
  "border border-[var(--cta-border)]",
  "rounded-[var(--radius-pill)]",
].join(" ");

const variantMap: Record<ButtonVariant, string> = {
  primary: primaryStyles,
  ghost: ghostStyles,
  spark: sparkStyles,
  dark: ["bg-[var(--text)]", "text-[var(--bg)]", "border border-transparent"].join(" "),
};

const sizeMap: Record<ButtonSize, string> = {
  sm: "px-4 py-2 text-xs",
  md: "px-5 py-2.5 text-sm",
  lg: "px-7 py-3.5 text-sm",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      className = "",
      variant = "primary",
      size = "md",
      isPill = true,
      isLoading = false,
      disabled,
      children,
      ...props
    },
    ref
  ) => {
    const baseStyles = [
      "inline-flex items-center justify-center",
      "font-medium",
      "tracking-wide",
      "transition-all duration-200",
      "ease-[cubic-bezier(0.22,1,0.36,1)]",
      "focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--bg)]",
      "disabled:opacity-50 disabled:cursor-not-allowed",
      "cursor-pointer",
      "whitespace-nowrap",
    ].join(" ");

    const radiusStyles = isPill
      ? "rounded-[var(--radius-pill)]"
      : "rounded-[var(--radius-button)]";

    return (
      <button
        ref={ref}
        className={[
          baseStyles,
          variantMap[variant],
          sizeMap[size],
          radiusStyles,
          className,
        ].join(" ")}
        disabled={disabled || isLoading}
        {...props}
      >
        {isLoading ? (
          <span className="flex items-center gap-2">
            <svg
              className="animate-spin h-4 w-4"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
            <span>Loading...</span>
          </span>
        ) : (
          children
        )}
      </button>
    );
  }
);

Button.displayName = "Button";