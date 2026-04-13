# Gunmetal Design System
## Inspired by SpaceX, x.ai, and the Gunmetal Brand Identity

---

## 1. Philosophy & Visual Theme

### Core Concept: "Something that came out of a gun"
Gunmetal is industrial, precise, powerful, and contained. When gunmetal sparks, there's a brief flash of energy — controlled, momentary, contained power. The design language reflects this: stark, minimal, technically precise, with moments of subtle brilliance.

### References
- **SpaceX**: Pure black canvas, stark typography, full-bleed imagery philosophy, minimal decoration
- **x.ai**: Stark monochrome, futuristic minimalism, pure black backgrounds, golden accents for CTAs
- **Gunmetal Brand**: The material itself — dark metallic, industrial, precise, powerful yet controlled

### Visual Atmosphere
The page is a void — pure black `#000000` like the emptiness of space. Against this void, white typography emerges with technical precision. A subtle blueprint grid suggests engineering rigor. Interactions feel deliberate and controlled — no bounce, no playfulness, just sharp precision.

**Key Characteristics:**
- Pure black canvas (`#000000`) — the void, minimal distraction
- White hierarchy (`#ffffff` → `#60646c` → `#3d3d3d`) — clear information priority
- Subtle blueprint grid (1.5% opacity, 64px cells) — engineering precision
- Minimal decoration — every element serves a purpose
- Sharp, technical animations — no bounce, no overshoot

---

## 2. Color Palette & Roles

### Background & Surface
| Token | Hex | Usage |
|-------|-----|-------|
| `--bg` | `#000000` | Page background — pure void |
| `--surface` | `#0a0a0a` | Cards, containers — gunmetal dark |
| `--surface-elevated` | `#141414` | Elevated elements — slight lift |

### Text Hierarchy
| Token | Hex | Usage |
|-------|-----|-------|
| `--text` | `#ffffff` | Primary text, headlines |
| `--text-secondary` | `#60646c` | Slate gray — secondary information |
| `--text-tertiary` | `#3d3d3d` | Muted — de-emphasized content |

### Borders & Lines
| Token | Hex | Usage |
|-------|-----|-------|
| `--border` | `#1a1a1a` | Subtle borders — barely visible |
| `--border-hover` | `#2a2a2a` | Border on hover — slightly revealed |

### Accent & Interactive
| Token | Hex | Usage |
|-------|-----|-------|
| `--accent` | `#0d74ce` | Links, functional accents — technical blue |
| `--accent-glow` | `rgba(13, 116, 206, 0.15)` | Subtle accent glow |
| `--accent-muted` | `#0a5fa8` | Accent hover state |

### CTA — "Spark" Effect
The CTA embodies gunmetal's spark — a brief flash of contained energy on an otherwise dark surface.

| Token | Hex | Usage |
|-------|-----|-------|
| `--cta-bg` | `#1a1a1a` | CTA button background — gunmetal dark |
| `--cta-text` | `#ffffff` | CTA button text — white |
| `--cta-border` | `#2a2a2a` | CTA button border |
| `--cta-glow` | `rgba(255, 255, 255, 0.06)` | Hover inner glow — spark flash |
| `--cta-hover-bg` | `#222222` | CTA button hover background |

### Grid & Blueprint
| Token | Value | Usage |
|-------|-------|-------|
| `--grid-color` | `rgba(255, 255, 255, 0.015)` | Blueprint grid lines |
| `--grid-size` | `64px` | Grid cell size |

---

## 3. Typography Rules

### Font Families
- **Primary**: `Inter` — clean, geometric, professional
- **Monospace**: `JetBrains Mono` — technical, precise
- **Fallback**: `-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`

### Type Scale

| Role | Size | Weight | Line Height | Letter Spacing | Notes |
|------|------|--------|-------------|----------------|-------|
| **Display (Hero)** | `clamp(3rem, 10vw, 5rem)` | 800 | 1.05 | -0.04em | Maximum impact, compressed |
| **Section** | `clamp(1.75rem, 4vw, 2.5rem)` | 600 | 1.10 | -0.02em | Clear section hierarchy |
| **Subheading** | `1.25rem` | 500 | 1.25 | -0.01em | Card titles, emphasis |
| **Body** | `1rem` | 400 | 1.40 | normal | Standard reading |
| **Caption** | `0.875rem` | 400 | 1.40 | normal | Descriptions |
| **Code** | `0.875rem` | 400 | 1.40 | normal | Monospace inline |
| **Label** | `0.75rem` | 500 | 1.00 | 0.08em | Uppercase, technical labels |
| **Step Number** | `3rem` | 700 | 1.00 | 0.02em | Monospace, large, muted |

### Typography Principles
- **Extreme tracking compression at scale**: Display headlines use -0.04em tracking — ultra-dense, feels like engineered logotype
- **Sharp hierarchy through weight contrast**: 800 for hero, 600 for section, 500 for emphasis, 400 for body
- **Technical precision over decoration**: No ornamental type treatment — every character serves communication
- **Inter at full weight range**: 400 (regular) through 800 (black) — unified voice with dramatic contrast

---

## 4. Component Styling

### Buttons — "Spark" CTAs

**Primary CTA (Install/Action)**
```css
background: var(--cta-bg);      /* #1a1a1a - gunmetal dark */
color: var(--cta-text);         /* #ffffff - white */
border: 1px solid var(--cta-border);  /* #2a2a2a */
padding: 14px 28px;
border-radius: 9999px;          /* Pill shape */
font-weight: 500;
letter-spacing: 0.02em;
transition: all 200ms cubic-bezier(0.22, 1, 0.36, 1);
```

**Hover State — The "Spark"**
```css
/* On hover: subtle inner glow — brief flash like gunmetal sparking */
box-shadow: inset 0 0 20px var(--cta-glow);
transform: scale(1.02);
background: var(--cta-hover-bg);
```

**Ghost/Secondary CTA**
```css
background: transparent;
color: var(--text-secondary);
border: 1px solid var(--border);
padding: 14px 28px;
border-radius: 9999px;
transition: all 200ms cubic-bezier(0.22, 1, 0.36, 1);
```

**Hover**: border-color brightens to `--border-hover`, text to `--text`

### Navigation

**Default State (Transparent)**
```css
background: transparent;
backdrop-filter: none;
border-bottom: none;
```

**Scrolled State**
```css
background: rgba(0, 0, 0, 0.7);
backdrop-filter: blur(20px);
-webkit-backdrop-filter: blur(20px);
border-bottom: 1px solid var(--border);
```

**Nav Structure**
- Height: 64px
- Logo: "GM" monogram left (white on dark, rounded-sm)
- Links: Products, Developer (center, 14px, 500 weight)
- Actions: GitHub button (right)

### Cards & Containers

**Dark Surface Card**
```css
background: var(--surface);     /* #0a0a0a */
border: 1px solid var(--border);  /* #1a1a1a */
border-radius: 8px;             /* Subtle rounding */
padding: 24px;
transition: all 200ms ease;
```

**Hover**
```css
border-color: var(--border-hover);  /* #2a2a2a */
transform: translateY(-2px);
```

### Inputs (Minimal)
```css
background: var(--surface);
border: 1px solid var(--border);
border-radius: 6px;
padding: 10px 14px;
color: var(--text);
transition: border-color 150ms ease;
```

**Focus**: border-color to `--accent`, subtle accent glow

### Terminal/Code Block
```css
background: var(--surface);     /* #0a0a0a */
border: 1px solid var(--border);
border-radius: 8px;
padding: 16px 20px;
font-family: var(--font-mono);  /* JetBrains Mono */
font-size: 0.875rem;
```

---

## 5. Layout Principles

### Spacing System
Base unit: **8px**

| Token | Value |
|-------|-------|
| `--space-1` | 4px |
| `--space-2` | 8px |
| `--space-3` | 12px |
| `--space-4` | 16px |
| `--space-6` | 24px |
| `--space-8` | 32px |
| `--space-12` | 48px |
| `--space-16` | 64px |
| `--space-24` | 96px |
| `--space-32` | 128px |

### Section Pacing
- **Hero**: Full viewport height (100vh) — maximum breathing room
- **Below Hero**: 96-128px vertical padding between sections — gallery-like pacing
- **Card grids**: 24px gap between cards
- **Content max-width**: 1200px, centered

### Grid System
- Blueprint grid: 64px cells, `rgba(255,255,255,0.015)` color
- Subtle parallax on scroll (0.05x speed) for depth
- Grid only visible on light content areas, fades against dark

### Border Radius Scale
| Element | Radius |
|---------|--------|
| Buttons (pill) | 9999px |
| Cards | 8px |
| Inputs | 6px |
| Small elements | 4px |
| Navbar | 0px (sharp) |

---

## 6. Animation System

### Philosophy
**Technical & Precise** — every animation serves communication, not decoration. No bounce, no overshoot, no playfulness. Animations feel like precision machinery: deliberate, controlled, efficient.

### Easing Curve
```css
cubic-bezier(0.22, 1, 0.36, 1)  /* Sharp, no bounce */
```

### Animation Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `--duration-fast` | 150ms | Hover states, micro-interactions |
| `--duration-normal` | 400ms | Standard transitions |
| `--duration-slow` | 700ms | Section reveals, major transitions |
| `--ease-out` | `cubic-bezier(0.22, 1, 0.36, 1)` | Primary easing |

### Key Animations

**1. Hero Label Fade-In**
```css
animation: fadeInUp 400ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
delay: 200ms;
opacity: 0 → 1;
transform: translateY(8px) → translateY(0);
```

**2. Hero Headline Word-by-Word Reveal**
```css
/* Each word fades in + slight Y translation, staggered 60ms */
animation: wordReveal 400ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
stagger: 60ms per word;
starts: 400ms delay;
```

**3. Hero Subheading Fade-In**
```css
animation: fadeIn 500ms ease-out forwards;
delay: 1000ms;
opacity: 0 → 1;
```

**4. Terminal Typewriter**
```css
animation: typewriter 800ms steps(45) forwards;
delay: 1200ms;
/* Characters appear one by one, 25ms each for ~45 chars */
```

**5. CTA Stagger Reveal**
```css
animation: fadeInScale 400ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
stagger: 150ms between buttons;
starts: 1600ms delay;
transform: scale(0.95) → scale(1);
```

**6. Scroll Indicator**
```css
animation: fadeIn 600ms ease-out forwards, bounce 2s ease-in-out infinite;
delay: 2000ms for fade-in;
bounce: translateY(0) → translateY(8px) → translateY(0), 2s loop;
```

**7. Section Scroll Reveal**
```css
animation: revealUp 700ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
trigger: IntersectionObserver at 10% visibility;
opacity: 0 → 1;
transform: translateY(24px) → translateY(0);
```

**8. Blueprint Grid Parallax**
```css
/* Background moves at 0.05x scroll speed for subtle depth */
transform: translateY(calc(var(--scroll) * 0.05));
```

### Keyframes

```css
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes fadeInScale {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

@keyframes wordReveal {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes typewriter {
  from { clip-path: inset(0 100% 0 0); }
  to { clip-path: inset(0 0 0 0); }
}

@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(8px); }
}
```

---

## 7. Page Structure — Streamlined

Four sections only. No clutter, no jargon, no unnecessary content.

```
┌─────────────────────────────────────────────────────────┐
│ NAVBAR                                                    │
│ [GM Logo]        [Products] [Developer]      [GitHub]   │
├─────────────────────────────────────────────────────────┤
│                                                          │
│                     HERO                                  │
│                   (100vh)                                │
│                                                          │
│         LOCAL AI INFERENCE GATEWAY                       │
│                                                          │
│    The middleman layer for AI inference.                  │
│                                                          │
│    Turn your AI subscriptions into one                     │
│    local API for inference.                              │
│                                                          │
│    $ npm install -g @dhruv2mars/gunmetal                 │
│                                                          │
│    [Install Gunmetal]  [Documentation]                   │
│                                                          │
│                        ▼                                  │
├─────────────────────────────────────────────────────────┤
│                                                          │
│                   HOW IT WORKS                            │
│                                                          │
│    01              02              03                     │
│    Mount           Mint            Direct                 │
│    Providers       Access          Apps                   │
│                                                          │
│    [desc]          [desc]          [desc]                │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│                   PROVIDERS                               │
│                                                          │
│    Codex  ·  GitHub Copilot  ·  OpenRouter  ·  Zen  ·  OpenAI
│                                                          │
├─────────────────────────────────────────────────────────┤
│ FOOTER                                                    │
│ [GM] Gunmetal                    [Docs] [GitHub] [©2026]│
└─────────────────────────────────────────────────────────┘
```

### Section Specifications

**1. Hero (100vh)**
- Full viewport, vertically centered
- Blueprint grid background (subtle parallax)
- Radial glow at bottom center (5% opacity, accent color)
- Word-by-word headline reveal
- Typewriter terminal effect
- Spark CTA buttons

**2. How It Works**
- Three numbered steps in horizontal layout (desktop) / vertical (mobile)
- Step numbers: `01`, `02`, `03` in monospace, 3rem, muted color
- Arrow connectors between steps (desktop only)
- Scroll reveal animation

**3. Providers**
- Minimal logo/icons in row
- Equal spacing, centered
- Hover: subtle brightness increase
- No borders, no cards — pure icon row

**4. Footer**
- Logo left, links center/right, copyright far right
- Minimal, SpaceX-inspired
- No decoration

---

## 8. Responsive Breakpoints

| Breakpoint | Width | Layout Changes |
|------------|-------|----------------|
| Mobile | <640px | Single column, hero text ~36px, stacked CTAs vertical |
| Tablet | 640-1024px | 2-column grids, medium hero |
| Desktop | >1024px | Full layout, max-width containers |

### Mobile Adaptations
- Hero headline: `clamp(2rem, 8vw, 3rem)` — scales down
- CTAs: Stack vertically, full-width buttons
- Sections: Reduce padding by ~40%
- Grid: Hidden on mobile (performance + aesthetics)
- Nav: Hamburger menu at <1024px

---

## 9. Do's and Don'ts

### Do
- Use pure black `#000000` as the canvas — the void is intentional
- Keep display headlines at -0.04em tracking — engineered compression
- Use spark CTAs with subtle inner glow on hover — contained energy
- Apply `cubic-bezier(0.22, 1, 0.36, 1)` for all animations — sharp, precise
- Create deliberate staggered reveals — technical, not whimsical
- Use blueprint grid at 1.5% opacity — engineering precision
- Let typography carry the visual weight — minimal decoration
- Target 100vh for hero — full viewport immersion

### Don't
- Don't use shadows for depth — use border-contrast and subtle backgrounds
- Don't add decorative colors — keep palette achromatic + one functional accent
- Don't use bounce or overshoot on animations — precision over playfulness
- Don't create card-heavy layouts — minimal containers, let content breathe
- Don't use more than 4 sections — restraint is design
- Don't mix in decorative imagery — text and geometry only
- Don't use sentence case on labels — uppercase technical labels (12px, 0.08em tracking)
- Don't create gradient backgrounds — solid colors only

---

## 10. Component Quick Reference

### Color Tokens
```css
--bg: #000000;                    /* Page background */
--surface: #0a0a0a;               /* Cards, containers */
--surface-elevated: #141414;      /* Elevated elements */
--text: #ffffff;                  /* Primary text */
--text-secondary: #60646c;        /* Slate gray */
--text-tertiary: #3d3d3d;         /* Muted */
--border: #1a1a1a;                /* Subtle borders */
--accent: #0d74ce;                /* Links, functional */
--cta-bg: #1a1a1a;               /* CTA background */
--cta-glow: rgba(255,255,255,0.06); /* Spark effect */
```

### Example Component Prompts

**Hero Headline**:
"Create a 5rem display headline in Inter 800 weight with -0.04em tracking and #ffffff color. Apply word-by-word reveal animation with 60ms stagger per word, starting 400ms after page load."

**Spark CTA Button**:
"Build a pill-shaped button (9999px radius) with #1a1a1a background and white text. On hover: inset box-shadow rgba(255,255,255,0.06), scale(1.02), 200ms transition with cubic-bezier(0.22, 1, 0.36, 1)."

**Blueprint Grid**:
"Add a background grid using linear-gradient with rgba(255,255,255,0.015) lines on 64px cells. Apply subtle parallax at 0.05x scroll speed."

**Step Number**:
"Display '01' in JetBrains Mono at 3rem weight 700 with #60646c color and 0.02em tracking. Add reveal animation on scroll with 100ms transition delay."

**Terminal Block**:
"Create a code block with #0a0a0a background, 1px solid #1a1a1a border, 8px radius, and JetBrains Mono 0.875rem. Apply typewriter animation starting 1.2s after page load."

---

## 11. Animation Timing Summary

| Element | Duration | Delay | Easing |
|---------|----------|-------|--------|
| Grid parallax | continuous | — | linear |
| Hero label | 400ms | 200ms | cubic-bezier(0.22, 1, 0.36, 1) |
| Headline words | 400ms each | +60ms stagger, starts 400ms | cubic-bezier(0.22, 1, 0.36, 1) |
| Hero subhead | 500ms | 1000ms | ease-out |
| Terminal typewriter | ~800ms | 1200ms | steps(45) |
| CTAs | 400ms | 1600ms + 150ms stagger | cubic-bezier(0.22, 1, 0.36, 1) |
| Scroll indicator | 600ms fade + 2s bounce loop | 2000ms | ease-out + ease-in-out |
| Section reveals | 700ms | 0ms (triggered) | cubic-bezier(0.22, 1, 0.36, 1) |

**Total hero reveal**: ~2.5s to fully visible

---

*Design system for Gunmetal — crafted with precision.*