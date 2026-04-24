# Gunmetal Landing Design

This file is the source of truth for the current landing page, navbar, footer, and brand direction in `apps/web`.

## Brand Identity

Gunmetal should feel calm, dark, precise, and quietly technical.

The landing page is intentionally minimal: one strong statement, one animated command surface, one restrained navbar, and a quiet footer. It should not feel like a broad SaaS dashboard, a colorful AI startup, or a generic docs template. It should feel like a local-first developer tool with confidence in its own simplicity.

Core traits:

- Warm near-black canvas.
- Off-white, not pure white, typography.
- Muted gray navigation and secondary text.
- Very little color.
- No purple/blue AI-gradient look.
- No noisy cards, product grids, or feature blocks on the landing page.
- One central visual idea: a command line install box.
- Polished micro-interactions over heavy decoration.

## Current Landing Page

Route: `/`

Current composition:

- Fixed top navbar.
- Centered hero section.
- H1: `The middleman layer for AI inference.`
- Subcopy: `Use AI subscriptions as upstream providers for inference.`
- Animated install command box.
- Minimal footer.

The page is allowed to be sparse. Empty space is part of the design. Do not add extra CTAs, cards, proof blocks, feature rows, diagrams, or screenshots unless explicitly requested.

## Landing Decisions

Keep:

- Current headline and subcopy.
- Current navbar structure and labels.
- Current footer simplicity.
- Current dark minimal atmosphere.
- Command box as primary landing interaction.
- Animated package-manager cycling in command box.
- Copy-to-clipboard behavior.

Do not add:

- Extra install CTAs under the command box.
- Secondary hero buttons.
- Marketing feature grids.
- New copy explaining the product in the landing hero.
- Visual assets or screenshots on the landing page.
- Bright accent colors.
- Gradients, blobs, or decorative orbs.

## Layout

Page shell:

- Full viewport dark surface.
- `Navbar` fixed at top.
- `Footer` sits at bottom when content is short.
- Main content uses flex center alignment.
- Hero content is horizontally centered.
- Landing max width follows site container rhythm: `max-w-7xl`, `px-6`, `lg:px-8`.

Hero:

- Main section: `flex-1 flex flex-col items-center justify-center`.
- Text alignment: center.
- H1 max width: about `17ch`.
- H1 line-height: `0.95`.
- H1 letter-spacing: `0`.
- H1 weight: `400`.
- Subcopy sits close under H1, with enough gap before command box.

Background:

- Warm dark base from `--bg`.
- One very subtle warm glow behind hero:
  - `rgba(250,249,246,0.02)`
  - large blurred oval
  - no visible colored gradient
  - no separate decorative blobs

## Typography

Primary landing font:

- `var(--font-matter)` for hero and nav.
- Current implementation maps this to `DM Sans`.
- Weight should stay mostly `400`.
- Brand/nav accents may use `500`.

Rules:

- No negative letter-spacing on landing H1.
- No bold display weight.
- Hero text should be large, quiet, and legible.
- Secondary text uses muted gray, not strong white.
- Code uses `var(--font-jetbrains)`.

Current key sizes:

- H1: `clamp(2.5rem, 8vw, 6.5rem)`.
- Subcopy: `18px`, `22px` from `md`.
- Command text: `14px` mono.
- Navbar links: `14px`.
- Footer text: `11px` to `16px`.

## Color

Use warm, restrained colors from `apps/web/src/app/globals.css`.

Primary tokens:

- Background: `--bg` = `#0e0e0d`.
- Primary text: `--text` = `#faf9f6`.
- Muted text: `--text-muted` = `#868584`.
- Subtle border: low-opacity `rgba(226,226,226,...)`.
- Frosted hover: `--frosted` = `rgba(255,255,255,0.04)`.

Color rules:

- Avoid pure white.
- Avoid blue/purple gradients.
- Avoid strong accent CTAs.
- Hover states should be opacity/color shifts within warm neutral scale.
- Borders should be subtle, usually below `0.18` alpha.

## Navbar

The navbar is part of the landing identity. Keep it.

Current structure:

- Left: logo mark + `Gunmetal`.
- Right desktop nav:
  - `Products`
  - `Developer`
  - `Resources`
  - `Download`
  - GitHub icon
- Mobile:
  - logo
  - GitHub icon
  - menu button
  - accordion nav

Current dropdown content:

- Products:
  - `Gunmetal Suite`
  - `The all-in-one platform for your needs.`
- Developer:
  - `Extension SDK`
  - `Build powerful native integrations.`
- Resources:
  - `Documentation`
  - `Guides and API references.`
  - `Changelogs`
  - `Latest updates and improvements.`

Navbar visual rules:

- Fixed top.
- Height: `56px`.
- Background: `rgba(14,14,13,0.70)`.
- Backdrop blur is allowed.
- One faint bottom border via box shadow.
- Nav text stays muted until hover/open.
- Dropdowns use warm dark translucent panels.
- Dropdowns may have subtle shadow, but no colorful glow.
- Keep labels and dropdown copy unless explicitly changed.

Known route state:

- Some navbar routes may not exist yet. Do not remove nav items only because route pages are pending.

## Command Box

The command box is the main landing-page object.

Purpose:

- Communicate install/start energy without adding more hero CTAs.
- Feel like a compact terminal command.
- Be clickable and copy the currently displayed command.

Current behavior:

- Cycles package manager labels:
  - `npm`
  - `bun`
  - `pnpm`
  - duplicate `npm` for seamless loop
- Tail remains:
  - `i -g  @dhruv2mars/gunmetal`
- Visible command should not show ellipsis on desktop.
- Copy command matches active package manager.
- Reduced-motion users do not get cycling animation.
- Clipboard write is wrapped in `try/catch`.
- Fallback copy uses temporary hidden textarea + `document.execCommand("copy")`.
- Show `Copied` only when copy succeeds.

Current visual:

- Dark translucent box.
- Subtle border.
- `12px` radius.
- Mono text.
- `$` prompt at left.
- Animated package manager slot.
- Copy icon at right.
- Max width: `500px`.
- No external CTA text.

Rules:

- Preserve animation concept.
- Preserve compact command-line feel.
- Do not replace with large button CTA.
- Do not add package-manager tabs unless requested.
- Do not let desktop command truncate.
- On very narrow mobile, graceful containment is acceptable; avoid layout overflow.

## Footer

Footer stays quiet.

Current structure:

- Left: logo + `Gunmetal`.
- Year: `© 2026`.
- Right: GitHub text link with small arrow.

Footer rules:

- Do not expand into a sitemap by default.
- Keep border subtle.
- Keep text muted.
- Match navbar warmth.

## Motion

Motion should be small and purposeful.

Allowed:

- Navbar logo text compressing on scroll.
- Dropdown fade/translate.
- Mobile accordion open/close.
- Command package-manager vertical loop.
- Copy success swap.
- Subtle icon hover translate in footer.

Rules:

- Use transform/opacity where possible.
- Respect `prefers-reduced-motion`.
- No bounce.
- No large page entrance animations.
- No background animation.

## Responsive Guidance

Desktop:

- Hero can be large and sparse.
- Command should show full visible command.
- Navbar shows full nav.

Mobile:

- Navbar collapses.
- H1 remains centered and legible.
- Command box may shrink, but must not break layout.
- Prefer horizontal padding over text collision.
- Mobile menu should feel like a full-height overlay panel if it is refined later.

Suggested future mobile fix:

- Change mobile menu panel to fixed overlay:
  - `fixed inset-x-0 top-14 bottom-0 overflow-y-auto`
- Avoid placing `min-h-screen` inside the header flow.
- Keep GitHub + menu button in mobile header.

## Accessibility

Current expectations:

- Skip link remains.
- Navbar has `aria-label="Primary"`.
- Dropdown buttons expose `aria-haspopup`, `aria-expanded`, and `aria-controls`.
- Dropdown links use `role="menuitem"`.
- Command box has descriptive `aria-label`.
- Copy success uses `aria-live`.
- Decorative logo images use empty alt + `aria-hidden`.

Do:

- Preserve keyboard access for dropdowns.
- Keep visible focus rings subtle but present.
- Avoid changing command animation in ways that confuse screen readers.

## Implementation Anchors

Landing files:

- `apps/web/src/app/page.tsx`
- `apps/web/src/components/ui/PackageManagerCommandBox.tsx`
- `apps/web/src/components/layout/Navbar.tsx`
- `apps/web/src/components/layout/Footer.tsx`
- `apps/web/src/app/globals.css`

Current verified behavior:

- Landing page served at `http://localhost:3000/`.
- Command box full command visible on desktop.
- Copy click does not throw an unhandled clipboard error.
- Web tests pass.
- Web lint passes with one existing Navbar `<img>` warning.

## Do / Do Not

Do:

- Keep landing sparse.
- Keep current copy.
- Keep navbar and footer as designed.
- Preserve command-box animation.
- Use warm neutral palette.
- Use crisp alignment and stable dimensions.
- Verify in browser after visual edits.

Do not:

- Rewrite brand copy without explicit request.
- Add explanatory sections to landing.
- Add new CTAs beside the command box.
- Replace navbar labels.
- Remove pending nav items because pages are not created yet.
- Add colorful accents, gradients, or visual clutter.
- Reintroduce negative H1 letter-spacing on the landing page.
