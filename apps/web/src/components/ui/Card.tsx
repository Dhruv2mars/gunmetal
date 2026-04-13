"use client";

import { type HTMLAttributes, forwardRef } from "react";

type CardVariant = "default" | "elevated" | "featured";
type CardPadding = "sm" | "md" | "lg";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  variant?: CardVariant;
  padding?: CardPadding;
  hoverable?: boolean;
}

/* ----------------------------------------------
   CARD STYLES — Expo Dark Adaptation

   Default: Surface bg, subtle border, NO shadow
     Shadow is only for hover (whisper shadow)

   Featured: Surface bg, border, elevated shadow
     Used for hero callouts and featured content
   ---------------------------------------------- */

const variantStyles: Record<CardVariant, string> = {
  default: [
    "bg-[var(--color-surface)]",
    "border border-[var(--color-border)]",
    "shadow-none",
  ].join(" "),
  elevated: [
    "bg-[var(--color-surface)]",
    "border border-[var(--color-border)]",
    "shadow-[var(--shadow-whisper)]",
  ].join(" "),
  featured: [
    "bg-[var(--color-surface)]",
    "border border-[var(--color-border)]",
    "shadow-[var(--shadow-elevated)]",
  ].join(" "),
};

/* Expo spec: Card padding 24-32px = p-6 to p-8 */
const paddingStyles: Record<CardPadding, string> = {
  sm: "p-4",
  md: "p-6",
  lg: "p-8",
};

export const Card = forwardRef<HTMLDivElement, CardProps>(
  (
    {
      className = "",
      variant = "default",
      padding = "md",
      hoverable = false,
      children,
      ...props
    },
    ref
  ) => {
    /* Hoverable: whisper shadow on hover, subtle lift */
    const hoverStyles = hoverable
      ? [
          "cursor-pointer",
          "transition-all duration-200",
          "hover:shadow-[var(--shadow-whisper)]",
          "hover:-translate-y-0.5",
        ].join(" ")
      : "";

    /* 8px radius per Expo card spec: "comfortably rounded (8px) for standard cards" */
    const baseRadius = "rounded-[var(--radius-card)]";

    return (
      <div
        ref={ref}
        className={[
          baseRadius,
          variantStyles[variant],
          paddingStyles[padding],
          hoverStyles,
          className,
        ].join(" ")}
        {...props}
      >
        {children}
      </div>
    );
  }
);

Card.displayName = "Card";

type CardHeaderProps = HTMLAttributes<HTMLDivElement>;

export const CardHeader = forwardRef<HTMLDivElement, CardHeaderProps>(
  ({ className = "", children, ...props }, ref) => (
    <div ref={ref} className={`mb-4 ${className}`} {...props}>
      {children}
    </div>
  )
);

CardHeader.displayName = "CardHeader";

/* Sub-heading: 20px, weight 600, -0.25px tracking, 1.20 lh per Expo spec */
type CardTitleProps = HTMLAttributes<HTMLHeadingElement>;

export const CardTitle = forwardRef<HTMLHeadingElement, CardTitleProps>(
  ({ className = "", children, ...props }, ref) => (
    <h3
      ref={ref}
      className={[
        "text-subheading",
        "text-[var(--color-text)]",
        className,
      ].join(" ")}
      {...props}
    >
      {children}
    </h3>
  )
);

CardTitle.displayName = "CardTitle";

type CardDescriptionProps = HTMLAttributes<HTMLParagraphElement>;

export const CardDescription = forwardRef<HTMLParagraphElement, CardDescriptionProps>(
  ({ className = "", children, ...props }, ref) => (
    <p
      ref={ref}
      className={[
        "text-caption",
        "text-[var(--color-text-secondary)]",
        "mt-2",
        className,
      ].join(" ")}
      {...props}
    >
      {children}
    </p>
  )
);

CardDescription.displayName = "CardDescription";
