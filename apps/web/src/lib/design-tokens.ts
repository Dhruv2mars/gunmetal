export const colors = {
  // Primary
  expoBlack: "#000000",
  nearBlack: "#1c2024",

  // Accent
  linkCobalt: "#0d74ce",
  legalBlue: "#476cff",
  widgetSky: "#47c2ff",
  previewPurple: "#8145b5",

  // Surface Light
  cloudGray: "#f0f0f3",
  pureWhite: "#ffffff",

  // Surface Dark
  widgetDark: "#1a1a1a",
  bannerDark: "#171717",

  // Neutrals
  slateGray: "#60646c",
  midSlate: "#555860",
  silver: "#b0b4ba",
  pewter: "#999999",
  lightSilver: "#cccccc",
  darkSlate: "#363a3f",
  charcoal: "#333333",

  // Semantic
  warningAmber: "#ab6400",
  destructiveRose: "#eb8e90",
  borderLavender: "#e0e1e6",
  inputBorder: "#d9d9e0",
  darkFocusRing: "#2547d0",

  // Semantic Colors
  green: "#34c759",
  greenDark: "#30d158",
} as const;

export const fonts = {
  sans: "var(--font-inter)",
  mono: "var(--font-jetbrains)",
} as const;

export const spacing = {
  1: "4px",
  2: "8px",
  3: "12px",
  4: "16px",
  6: "24px",
  8: "32px",
  10: "40px",
  12: "48px",
  16: "64px",
  20: "80px",
  24: "96px",
  32: "128px",
  144: "144px",
} as const;

export const radius = {
  subtle: "4px",
  button: "6px",
  card: "8px",
  panel: "16px",
  media: "24px",
  nav: "32px",
  pill: "36px",
  full: "9999px",
} as const;

export const shadows = {
  whisper:
    "rgba(0,0,0,0.08) 0px 3px 6px, rgba(0,0,0,0.07) 0px 2px 4px",
  elevated:
    "rgba(0,0,0,0.1) 0px 10px 20px, rgba(0,0,0,0.05) 0px 3px 6px",
} as const;

export const typography = {
  display: {
    fontSize: "clamp(2.5rem, 8vw, 4rem)",
    fontWeight: 700,
    lineHeight: 1.1,
    letterSpacing: "-0.03em",
  },
  sectionHeading: {
    fontSize: "clamp(2rem, 5vw, 3rem)",
    fontWeight: 600,
    lineHeight: 1.1,
    letterSpacing: "-0.02em",
  },
  subheading: {
    fontSize: "1.25rem",
    fontWeight: 600,
    lineHeight: 1.2,
    letterSpacing: "-0.01em",
  },
  bodyLarge: {
    fontSize: "1.125rem",
    fontWeight: 400,
    lineHeight: 1.4,
  },
  body: {
    fontSize: "1rem",
    fontWeight: 400,
    lineHeight: 1.4,
  },
  caption: {
    fontSize: "0.875rem",
    fontWeight: 400,
    lineHeight: 1.4,
  },
  label: {
    fontSize: "0.75rem",
    fontWeight: 500,
    lineHeight: 1.2,
    letterSpacing: "0.05em",
    textTransform: "uppercase" as const,
  },
  code: {
    fontFamily: "var(--font-jetbrains)",
    fontSize: "0.875rem",
    fontWeight: 400,
    lineHeight: 1.6,
  },
} as const;

export const breakpoints = {
  mobile: "640px",
  tablet: "1024px",
  desktop: "1280px",
} as const;
