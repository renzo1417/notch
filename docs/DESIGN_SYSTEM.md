# PROJECT_NOTCH — Design System & Tokens

**Status:** Phase 0 Technical Foundation  
**Reference Document:** [PRODUCT_SPEC.md](../PRODUCT_SPEC.md)

---

## 1. Design Philosophy

`PROJECT_NOTCH` is an original Windows desktop companion designed to be **quiet by default, expressive on interaction, and light enough to run all day**.

It does not mimic mobile operating systems, browser widgets, or generic SaaS web applications. Every visual element has a functional purpose.

---

## 2. Anti-AI-Slop Rules

To maintain high visual craft, the following anti-patterns are strictly prohibited:

- **No Nested Cards:** Do not stack cards inside cards inside cards.
- **No Excessive Glassmorphism:** Avoid blurry transparent sheets that reduce readability.
- **No Purple/Blue AI Gradients:** Avoid trendy multicolor gradient meshes.
- **No Neon Borders:** Avoid glowing outlines and saturated neon borders.
- **No Emoji Icons:** Use consistent vector SVGs, never emojis as functional UI controls.
- **No Arbitrary Padding:** Spacing adheres strictly to the defined scale.
- **No Meaningless Badges or Sparkles:** Do not decorate elements without clear informational value.
- **No Gratuitous Animation:** Motion communicates state changes or tactile feedback, not continuous idle distraction.

---

## 3. Design Tokens

### 3.1 Surface Palette
```css
/* Dark theme base palette */
--surface-root: rgba(12, 12, 13, 0.96);     /* Main shell background */
--surface-raised: rgba(27, 27, 29, 0.96);   /* Elevated controls */
--surface-hover: rgba(255, 255, 255, 0.07); /* Hover states */
--surface-active: rgba(255, 255, 255, 0.12);/* Active / pressed states */
```

### 3.2 Typography Colors
```css
--text-primary: rgba(255, 255, 255, 0.96);   /* Titles and main values */
--text-secondary: rgba(255, 255, 255, 0.64); /* Metadata, labels */
--text-tertiary: rgba(255, 255, 255, 0.42);  /* Status hints, dividers */
```

### 3.3 Accent Color
```css
/* Restrained Windows Slate Blue */
--accent-primary: #3b82f6;
--accent-muted: rgba(59, 130, 246, 0.15);
--accent-focus: rgba(59, 130, 246, 0.40);
```

### 3.4 Borders & Shadows
```css
--border-subtle: rgba(255, 255, 255, 0.08);
--border-strong: rgba(255, 255, 255, 0.16);
--shadow-soft: 0 8px 24px rgba(0, 0, 0, 0.30);
```

### 3.5 Radius Scale
```css
--radius-shell: 18px;    /* Notch outer corner radius */
--radius-control: 8px;   /* Buttons, inputs, chips */
--radius-pill: 9999px;   /* Circular buttons and status indicators */
```

### 3.6 Spacing Scale
The spacing system strictly follows:
`4px, 6px, 8px, 12px, 16px, 20px, 24px`

```css
--space-4: 4px;
--space-6: 6px;
--space-8: 8px;
--space-12: 12px;
--space-16: 16px;
--space-20: 20px;
--space-24: 24px;
```

### 3.7 Typography System
- **Font Stack:** `"Segoe UI Variable", "Segoe UI", Inter, system-ui, -apple-system, sans-serif`
- **Sizes:**
  - Track / Timer / Main: `13px - 14px semibold`
  - Primary Labels: `12px - 13px medium`
  - Metadata / Subtitles: `10px - 12px normal`
  - Tiny Status: `10px - 11px normal`

---

## 4. Motion Guidelines

Motion communicates discrete state transitions rather than ambient decoration.

```css
--motion-micro: 120ms;    /* Button presses, hover feedback */
--motion-shell: 220ms;    /* Expand / collapse shell animations */
--motion-context: 180ms;  /* Context zone transitions */
--motion-mascot: 350ms;   /* Mascot glance / reaction transitions */

--ease-natural: cubic-bezier(0.16, 1, 0.3, 1);
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
```

Reduced motion preferences (`prefers-reduced-motion: reduce`) clamp all timing variables to `0ms`.

---

## 5. Three-Zone Expanded Layout Architecture

The notch expanded shell (520px wide by 76px tall) is partitioned into three purposeful zones:

```text
┌──────────────────────────────────────────────────────────────────┐
│   LEFT (Mascot)   │       CENTER (Context)       │ RIGHT (Music) │
│       ~96px       │            ~248px            │     176px     │
└──────────────────────────────────────────────────────────────────┘
```

### 5.1 Left Zone: Procedural Companion (~96px, 18–20% visual balance)
- Houses procedural 2D Canvas companion "Pip" (64px canvas, 72px cell).
- Interactive hover and click dynamics (squash, gaze tracking, mood reactions).
- Separated optically by subtle vertical divider `--border-subtle`.

### 5.2 Center Zone: Context Surface (~248px, 42–46% visual balance)
- Flexible container (`flex: 1`, `min-width: 0`) resilient to long titles.
- Strict typography scale:
  - Header badge / context category: `10px` uppercase semibold (`--text-tertiary`).
  - Primary title: `13px` semibold (`--text-primary`), single line ellipsis.
  - Context detail: `11px` normal (`--text-secondary`), single line ellipsis.
- Bounded on both sides with 1px `--border-subtle` dividers.
- No fake charts, no cards-in-cards, no decorative blobs.

### 5.3 Right Zone: Media Module Placeholder (176px, 34–36% visual balance)
- Fixed geometry: never resizes or jitters regardless of track status.
- Left slot: 38×38px artwork container (`--surface-raised`, 8px radius, vector music icon).
- Right slot: Truncated track title (12px semibold), artist/source (10px tertiary), and vector playback transport controls row (Previous, Play/Pause, Next) subdued when idle.

### 5.4 Compact Surface (300px × 38px)
- Left: Mini mascot avatar (26×26px canvas).
- Center: Immediate status summary (`--font-size-label` semibold).
- Right: Subdued 13px vector music indicator.

