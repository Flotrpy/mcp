# UI research: which guidance wins, and where

These are the sources reviewed for the website and dashboard. Where they disagree, the decision and the reason are recorded below. *Applied* means the decision is implemented in `site/`.

| # | Source | What it is |
|---|---|---|
| A | [Apple Design Resources](https://developer.apple.com/design/resources/) | Official UI kits (iOS/macOS 27), SF Pro / SF Mono / New York fonts, SF Symbols |
| B | [Apple UI Design Dos and Don'ts](https://developer.apple.com/design/tips/) | 10 fundamentals: 44 × 44 pt hit targets, 11 pt minimum text, contrast, spacing, alignment, controls near content |
| C | [Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines) (Materials, Color, Layout, Typography, Charts) | Liquid Glass vs standard materials, light/dark/increased-contrast colour, progressive disclosure, 17 pt body text, chart anatomy |
| D | [Emil Kowalski, *apple-design* skill](https://www.ui-skills.com/skills/emilkowalski/apple-design) | WWDC *Designing Fluid Interfaces* translated for the web: response, 1:1 tracking, interruptibility, springs, materials, reduced motion and transparency, size-specific tracking |
| E | [Apple UI kit (Figma, recreated)](https://www.figma.com/community/file/1234845281498117038/apple-ui-free-ui-kit-recreated) | The apple.com website patterns: thin global nav, product tiles with headline, subhead and two chevron links, alternating light and dark full-bleed bands |
| F | Dribbble [apple-ui](https://dribbble.com/tags/apple-ui) / [apple-website](https://dribbble.com/tags/apple-website) | Community shots. Dribbble blocks crawlers (HTTP 202 challenge), so these trends come from search summaries: bento grids, "Liquid Glass" glassmorphism, monochrome-plus-one-accent palettes |
| G | Vercel Web Interface Guidelines (used earlier) | Web-specific rules: CSS before JS, never `transition: all`, URL as state, focus, APCA contrast |

## Comparisons and decisions

### 1. Glass: everywhere (F) or only on functional chrome (C, D)?
- **F** shows glassmorphism on every card, which is the main trend in "Liquid Glass" shots.
- **C** is explicit: *"Don't use Liquid Glass in the content layer… use it sparingly"*. Glass is for the floating functional layer: navigation, toolbars and sheets.
- **D** agrees, and adds that you should never stack a light translucent surface on another.

**Decision: C/D.** Glass appears only on the navigation bar and the command palette. Server tiles, the dashboard and the terminal are solid. *Applied:*
- **Nav top edge:** a bright highlight along the top edge of the nav (D §12).
- **Scroll edge:** a soft fade replaces the hard hairline under the nav (D: "scroll edge effects, not hard dividers").
- **`prefers-reduced-transparency`:** glass turns solid.

### 2. Motion: CSS transitions (G) or springs (D)?
- **G** says to prefer CSS, then WAAPI, then JS.
- **D** says to avoid CSS transitions for anything *gesture-driven*, because they can't be grabbed mid-flight. Springs animate from the current value and carry velocity.

**Decision: both, split by input.**
- **State changes** (hover, tab switch, reveal, palette) use CSS transitions with ease-out and durations ≤ 300 ms.
- **Direct manipulation** uses a small velocity-aware spring in `requestAnimationFrame` that starts from the on-screen value and can be re-targeted mid-motion. This is the shelf you drag in the sag widget.
- **Reduced motion** replaces the spring with an instant set, keeping only opacity fades (D §14, G).

### 3. Hit targets: 24 px (G) or 44 pt (B)?
- **G** allows 24 px on desktop and 44 px on mobile.
- **B** says 44 × 44 pt everywhere.

**Decision:** 44 px whenever the primary pointer is coarse (`@media (pointer: coarse)`), and at least 32 px with a padded hit area on desktop. Apple's own website uses compact controls on desktop. *Applied.*

### 4. Typography: one tracking value or size-specific tracking (C, D)?
**C** and **D** agree that tracking depends on size: negative for display text, roughly zero for body, slightly positive for small text. Line height runs inversely to size. Use the system font (SF) and avoid light weights.

**Decision:** adopt it as tokens.
- **Display:** −0.035em at 1.02 leading.
- **Headings:** −0.022em.
- **Body:** 17 px (the HIG default) at −0.011em and 1.47 leading.
- **Small text (≤ 13 px):** +0.01em.
- **Weights:** never below 400.
- **Optical sizing:** `font-optical-sizing: auto`.

*Applied.*

### 5. Layout: bento (E, F) and progressive disclosure (C)
- **E/F** use the bento grid as the canonical "Apple" layout, which traces back to Apple's spec-grid product videos.
- **C** asks for progressive disclosure: show the common path first and put advanced detail one level deeper.

**Decision:** the bento stays.
- **Tiles:** each one says *one* thing plus three concrete facts.
- **Depth:** detail lives one click away, behind chevron links (E's "Learn more ›" pattern) to each README and the dashboard.
- **Band:** one full-bleed dark band (E's alternating bands) holds the interactive sag demo, so the only "product shot" on the page is something you can actually use.

### 6. Colour: brand accents vs meaning (C)
- **C** says: *"Avoid using the same color to mean different things,"* and asks for light, dark *and increased-contrast* variants.
- **F's** monochrome-plus-one-accent look is attractive, but five servers need five identities.

**Decision:**
- **Server accents are identity only.** Status always uses a separate green/amber/red scale and is always paired with a text label (G: "don't rely on colour alone").
- **Increased contrast:** `prefers-contrast: more` swaps in solid surfaces, stronger hairlines and darker secondary ink.

*Applied.*

### 7. Spatial consistency (D) and feedback on press (D, B)
- **Palette:** the command palette scales in from its trigger (the nav search button) and exits along the same path. Keyboard opening (⌘K) is instant, with no animation, following Emil's rule for frequent keyboard actions.
- **Buttons:** they respond on pointer-down (`:active` scale 0.97, 100 ms).

### 8. Charts for the dashboard (C) plus accessible palettes (G)
**C:**
- Highlight *a few key pieces*.
- Give axes and ticks only where they help reading.
- Describe every chart in words.

**G:**
- Use colour-blind-friendly colours.
- Use tabular numbers.

**Decision:**
- **Charts are hand-built SVG:** grade rings, token-cost bars per tool and lockfile status.
- **Descriptions:** each chart carries an `aria-label` sentence and a `<details>` data table.
- **Bars:** one hue at a time, with values printed next to each.

### Rejected
- **Glass content cards (F)**, because of C's content-layer rule.
- **Purple gradient blobs and aurora backgrounds (F)**, which are noise with no information.
- **Autoplaying carousels**, because G and C say motion should be input-driven and over-feedback trains people to ignore it (D §13).
- **Custom display fonts**, because A, C and D all say to use the system font first, and it ships optical sizing.
- **Haptics and sound**, since D's utility rule says they add nothing on a documentation site.
