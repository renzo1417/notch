# PROJECT_NOTCH — Design System & Tokens

**Status:** Matte Blue companion
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
--surface-root: #151618;     /* Main shell background */
--surface-raised: #202226;   /* Elevated controls */
--surface-hover: rgba(255, 255, 255, 0.07); /* Hover states */
--surface-active: rgba(255, 255, 255, 0.12);/* Active / pressed states */
```

### 3.2 Typography Colors
```css
--text-primary: #eceef1;   /* Titles and main values */
--text-secondary: #adb1b8; /* Metadata, labels */
--text-tertiary: #858a94;  /* Status hints, dividers */
```

### 3.3 Accent Color
```css
/* Matte Blue */
--accent-primary: #83acee;
--accent-muted: rgba(131, 172, 238, 0.15);
--accent-focus: #83acee;
```

### 3.4 Borders & Shadows
```css
--border-subtle: rgba(255, 255, 255, 0.08);
--border-strong: rgba(255, 255, 255, 0.16);
--shadow-soft: 0 6px 16px rgba(0, 0, 0, 0.24);
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

## 5. Focused drawer layout

| Surface | Logical size | Content |
|---|---|---|
| Compact | 304 × 44 | 32px Pip, immediate context, optional active-media indicator |
| Home | 480 × 160 | 40px Pip, activity summary, available media controls, navigation |
| Drawer | 480 × 328 | 56px companion header, full-width scrollable tool, navigation |
| Settings | 640 × 456 | Appearance / Tools / System tabs, scrollable grouped settings |

Use square top corners, 18px bottom corners, and 8px controls. Icons are normally 16px, with at least 32px hit areas. Apps use four columns; clipboard/download content uses flat rows and subtle separators. Pip uses the shared TypeScript palette: graphite chassis, dark visor, white eyes, yellow warnings, and no glossy highlight or glow.

`palette.ts` is shared by live appearance tokens and the canvas renderer. CSS provides matching startup defaults. Custom filled controls choose a contrasting foreground. Warning and error tokens remain separate from the configurable accent.

## 6. Interaction and accessibility

Click opens home; hover only changes Pip's expression. More opens an inline drawer, so secondary navigation cannot clip outside the shell. Back/Escape closes one level and restores focus. Explicit drawers do not react to background context navigation. Warnings/completions update only the header.

Use native buttons, visible focus indicators, keyboard tabs with arrow navigation, and a focus-contained settings dialog. Running countdowns are not live-announced every second. Honor both the OS reduced-motion preference and the app setting for CSS and Pip.

Keep 16px transparent margins at each side and below. Grow native bounds first; shrink after the effective transition. Clamp to monitor logical dimensions and scroll constrained content.


## 7. Matte Blue navigation and status

The matte shell is opaque by default. Translucent is opt-in; its slider retains the saved opacity when the user switches styles. Selection and focus use muted blue #83ACEE over neutral surfaces. Advisory #E3C56F, urgent attention #DFA16C and errors #E38F8F always accompany text/icons rather than communicating through color alone. Avoid gradients, gloss and luminous borders.

Primary navigation is Apps / Focus / Capture / More, with visible labels and a blue active state. One Back button in the shared header replaces repeated drawer close icons; Collapse remains explicitly available. More children return to More; the app picker returns to its opening Apps/Settings view. Search fields receive focus when the app picker opens. Inactive panels keep input and scroll state, and invisible Pip rendering is suspended by unmounting its header instance.

Installed apps uses searchable flat rows with icons and Add/Added states. Capture uses three labeled mode buttons, a target selector, a specific action (Select area / Capture window / Capture screen), and an inline success or error message. Both fit the 480 × 328 drawer and scroll when constrained. Capture disables navigation until completed or cancelled; status confirms copying only after native clipboard transfer succeeds.
