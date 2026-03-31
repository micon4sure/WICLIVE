# WICGATE Visual Design System - Comprehensive Style Guide

## Executive Summary

WICGATE uses a **military-inspired, dark-themed design system** based on the Massgate (World in Conflict) aesthetic. The design leverages a **graphite/steel color palette** with **orange accent colors** and implements a **zero-radius (sharp, squared) design language**. The system is built on CSS custom properties in the legacy system and Tailwind CSS in the current Nuxt website.

---

## COLOR SYSTEM

### Core Color Palette

#### **Background Colors** (Darkest Layer)
| Variable | Hex | Use Case |
|----------|-----|----------|
| `--bg` | `#050a0f` | Main page background (deepest military backdrop) |
| `--s1` | `#0e1a22` | Primary panel surface |
| `--s2` | `#14222a` | Secondary surface |
| `--surface-alt` | `#1f313d` | Alternating striping surface (tables, lists) |
| `texture.dark` (Tailwind) | `#0a0a0a` | Site background (current) |
| `texture.panel` (Tailwind) | `#151515` | Card/panel backgrounds |
| `texture.lighter` (Tailwind) | `#1e1e1e` | Elevated elements |

#### **Steel/Graphite Tones** (Structural Base)
| Variable | Hex | Use Case |
|----------|-----|----------|
| `--mg` | `#1f2f3b` | Primary steel tone (base structural color) |
| `--mg-dark` | `#101a22` | Deep steel (darker variant) |
| `--mg-muted` | `#344654` | Mid steel accent (subtle highlights) |
| `--graphite` | `#0f1215` | Dark graphite navigation base |
| `--graphite-dark` | `#08090b` | Even deeper graphite |
| `--graphite-light` | `#1a1e22` | Darker lighter graphite variant |
| `--ink` | `#0b141a` | Deepest charcoal ink (rare) |

#### **Accent Colors** (Primary Actions & Highlights)

**Massgate Orange** (CTA, hover, emphasis)
| Variable | Hex | Use Case |
|----------|-----|----------|
| `--sw` | `#f37c2b` | Massgate orange accent (primary action) |
| `--sw-light` | `#f8a85e` | Lighter accent highlight |
| `soviet` (Tailwind) | `#ff6600` | Orange for CTAs, downloads (current) |
| `soviet.light` | `#ff8533` | Light orange hover state |
| `soviet.dark` | `#e65c00` | Darker orange active state |

**Deep Soviet Red** (Critical/Important)
| Variable | Hex | Use Case |
|----------|-----|----------|
| `--dl` | `#8b1d14` | Deep Soviet red accent (alert/critical) |
| `--dl-light` | `#dc3545` | Alert highlight |
| `--dl-mid` | `#c82333` | Alert mid-tone |
| `--dl-dark` | `#a71e2a` | Alert deep (pressed) |
| `massgate-red` (Tailwind) | `#B22222` | Firebrick (important/critical) |
| `massgate-red.dark` | `#8B1A1A` | Critical elements |
| `massgate-red.bright` | `#DC2626` | Alerts & warnings |

**Medals & Status** (Ranking/Achievement)
| Variable | Hex | Use Case |
|----------|-----|----------|
| `--medal-gold` | `#ffd700` | 1st place (leaderboard) |
| `--medal-silver` | `#c0c0c0` | 2nd place (leaderboard) |
| `--medal-bronze` | `#cd7f32` | 3rd place (leaderboard) |
| `gold` (Tailwind) | `#ffd700` | Gold medal/achievement |
| `silver` (Tailwind) | `#c0c0c0` | Silver medal |
| `bronze` (Tailwind) | `#cd7f32` | Bronze medal |

**Discord & Brand Colors**
| Variable | Hex | Use Case |
|----------|-----|----------|
| `--d` | `#4f7bd7` | Discord brand blue (base) |
| `--d-light` | `#6b7ff3` | Discord highlight |
| `--d-dark` | `#4752c4` | Discord deep |
| `--d-darker` | `#3c4399` | Discord pressed |
| `discord` (Tailwind) | `#5865F2` | Discord official brand |
| `--brand-youtube` | `#e53935` | YouTube brand red |
| `--brand-youtube-bright` | `#ff5722` | YouTube bright |
| `--brand-youtube-peak` | `#ff0000` | YouTube peak red |
| `--brand-twitch` | `#9146ff` | Twitch brand purple |
| `--brand-twitch-dark` | `#7a38d8` | Twitch dark |

**Status Indicators**
| Variable | Hex | Use Case |
|----------|-----|----------|
| `--g` | `#7cb342` | Online/success indicator (bright green) |
| `--clan-tag` | `#b3b3b3` | Clan tag neutral gray |
| `--player-neutral` | `#dddddd` | Player list fallback |
| `online` (Tailwind) | `#7cb342` | Online status |

#### **Text Colors** (Typography Layer)
| Variable | Hex | Use Case |
|----------|-----|----------|
| `--t` | `#f3f6f8` | Primary text (bright near-white) |
| `--t2` | `#a7b7c3` | Secondary text (muted gray-blue) |
| `--t3` | `#6c7a85` | Tertiary text (dim gray) |
| `t` (Tailwind) | `#ffffff` | Primary text (pure white) |
| `t.secondary` | `#c5d5e0` | Secondary text (brighter gray-blue) |
| `t.dim` | `#8a9aa8` | Dimmed text |
| `t.tertiary` | `#6c7a85` | Tertiary text (muted) |

#### **Border & Divider Colors** (Structural Lines)
| Variable | Hex / RGBA | Use Case |
|----------|-----------|----------|
| `--bd` | `rgba(68, 117, 146, 0.25)` | Standard border (25% opacity) |
| `--divider-strong` | `rgba(68, 117, 146, 0.7)` | Strong dividers (70% opacity) |
| `--divider-soft` | `rgba(68, 117, 146, 0.35)` | Soft dividers (35% opacity) |
| `divider` (Tailwind) | `#447592` | Steel blue divider (base) |
| `divider.strong` | `rgba(68, 117, 146, 0.7)` | Strong borders |
| `divider.soft` | `rgba(68, 117, 146, 0.35)` | Soft borders |

### RGB Helper Variables (for use with rgba())

Legacy system provides pre-converted RGB values for opacity manipulation:
- `--mg-rgb: 31, 47, 59;` → `rgba(var(--mg-rgb), 0.5)` = semi-transparent steel
- `--sw-rgb: 243, 124, 43;` → `rgba(var(--sw-rgb), 0.3)` = semi-transparent orange
- `--dl-rgb: 139, 29, 20;` → `rgba(var(--dl-rgb), 0.2)` = semi-transparent red
- `--t-rgb: 243, 246, 248;` → `rgba(var(--t-rgb), 0.8)` = semi-transparent white
- Similar pattern for all major colors

---

## TYPOGRAPHY SYSTEM

### Font Families

| Family | Font Stack | Use Case | Notes |
|--------|-----------|----------|-------|
| **Military** (Headings) | `'Oswald', 'Impact', sans-serif` | H1-H4, buttons, nav | Bold, uppercase-friendly, commanding |
| **Body** (Content) | `'Rajdhani', sans-serif` | Body text, paragraphs, descriptions | Technical, geometric, readable |
| **Monospace** (Data) | `'Courier New', 'Monaco', monospace` | Scores, stats, player names, clan tags | Fixed-width, game-like |
| **Futuristic** | `'Orbitron', sans-serif` | Special emphasis (current Nuxt) | Decorative, rare use |

### Heading Styles

#### **H1** - Main Page Title
- Font Family: `Oswald` / `Impact`
- Font Size: `clamp(2.5rem, 6vw, 4.5rem)` (40px - 72px fluid)
- Font Weight: 700
- Line Height: 1.1
- Text Transform: `uppercase`
- Letter Spacing: 1px
- Text Shadow: 
  - `0 2px 10px rgba(0, 0, 0, 0.8)` (depth)
  - `0 0 20px rgba(var(--mg-rgb), 0.3)` (steel glow)
- Margin Bottom: 24px

#### **H2** - Section Headers
- Font Family: `Oswald` / `Impact`
- Font Size: `clamp(2rem, 4vw, 2.5rem)` (32px - 40px fluid)
- Font Weight: 600
- Text Transform: `uppercase`
- Letter Spacing: 0.5px
- Text Shadow: `0 2px 8px rgba(0, 0, 0, 0.7)`
- Margin Bottom: 20px

#### **H3** - Subsection Headers
- Font Family: `Rajdhani`
- Font Size: 1.75rem (28px)
- Font Weight: 700
- Text Transform: `uppercase`
- Letter Spacing: 0.5px
- Color: `var(--sw)` (orange)
- Margin Bottom: 16px

#### **H4** - Card/Panel Headers
- Font Family: `Rajdhani`
- Font Size: 1.25rem (20px)
- Font Weight: 600
- Text Transform: `uppercase`
- Letter Spacing: 0.3px
- Margin Bottom: 12px

#### **H5** - Subsection Title
- Font Family: `Rajdhani`
- Font Size: 1.125rem (18px)
- Font Weight: 600
- Color: `var(--sw)` (orange)
- Margin Bottom: 10px

### Body Text Styles

| Class | Font Size | Font Weight | Color | Use Case |
|-------|-----------|-----------|-------|----------|
| **p** (default) | 1rem | 400 | `--t` | Standard paragraph text |
| **.text-sm** | 0.875rem (14px) | 500 | `--t` | Small text, labels |
| **.text-xs** | 0.75rem (12px) | 500 | `--t` | Extra small (captions, badges) |
| **.text-muted** | 1rem | 400 | `--t2` | Secondary/muted text |
| **.text-dim** | 1rem | 300 | `--t3` | Dimmed tertiary text |
| **.military-data** | 1rem | 600 | `--t` | Stats, scores (monospace) |
| **.player-name** | 1rem | 600 | `--t` | Player display (monospace) |
| **.text-military** | 1rem | 600 | `--t` | Military text (uppercase, 1px letter spacing) |
| **.text-command** | 1rem | 700 | `--sw` | Command emphasis (orange, glow) |
| **.text-tactical** | 1rem | 600 | `--sw` | Tactical text (uppercase) |

### Specialty Typography

#### **Navigation Text** (`.nav-text`)
- Font Family: `Oswald`
- Font Weight: 500
- Text Transform: `uppercase`
- Letter Spacing: 1px

#### **Table Headers** (`.table-header`)
- Font Family: `Oswald`
- Font Weight: 600
- Text Transform: `uppercase`
- Letter Spacing: 1px
- Color: `var(--sw)` (orange)
- Text Shadow: `0 1px 3px rgba(0, 0, 0, 0.45)`

#### **Leaderboard Typography**

| Class | Font Size | Font Weight | Color | Notes |
|-------|-----------|-----------|-------|-------|
| **.leaderboard-title** | 1.5rem | 700 | `--sw` | Oswald, uppercase, 1px letter spacing |
| **.leaderboard-subtitle** | Small (0.85rem) | 500 | `--t2` | Rajdhani, uppercase, 0.5px spacing |
| **.rank-number** | 1.2rem | 700 | `--sw` | Oswald, text shadow |
| **.player-display** | 1rem | 600 | `--t` | Rajdhani, 0.3px letter spacing |
| **.rank-1 .lb-position** | 1.2rem | 700 | `var(--medal-gold)` | Gold color, shadow |
| **.rank-2 .lb-position** | 1.15rem | 700 | `var(--medal-silver)` | Silver color, shadow |
| **.rank-3 .lb-position** | 1.1rem | 700 | `var(--medal-bronze)` | Bronze color, shadow |

#### **Status Indicators**

| Class | Color | Weight | Style |
|-------|-------|--------|-------|
| **.status-online** | `--g` (green) | 600 | Oswald, uppercase, text glow |
| **.status-offline** | `--t3` (dim) | 500 | Rajdhani, uppercase |

#### **Fluid Typography Scale** (Tailwind Current)
The current Nuxt site uses a fluid typography system with clamp():
- **Hero**: `clamp(1.75rem, 1.607rem + 0.714vw, 2.25rem)` → 28px to 36px
- **Hero Secondary**: `clamp(1.25rem, 1.143rem + 0.536vw, 1.625rem)` → 20px to 26px
- **Heading**: `clamp(1.125rem, 0.970rem + 0.663vw, 1.5rem)` → 18px to 24px
- **Data**: `clamp(0.9375rem, 0.866rem + 0.357vw, 1.1875rem)` → 15px to 19px
- **Label**: `clamp(0.8125rem, 0.777rem + 0.179vw, 0.9375rem)` → 13px to 15px

---

## SPACING & LAYOUT

### Padding System

| Size | Value | Contexts |
|------|-------|----------|
| **xs** | 8px | Button inner (icon buttons) |
| **sm** | 10px - 12px | Tight spacing within components |
| **md** | 16px - 20px | Panel headers, card padding |
| **lg** | 24px - 32px | Section padding, hero spacing |
| **xl** | 40px - 60px | Large section gaps |

### Gap System (Flex/Grid)

| Size | Value | Use Case |
|------|-------|----------|
| **sm** | 10px | `.gap-sm` - tight lists |
| **md** | 20px | `.gap-md` - standard spacing |
| **Standard Grid** | 30px | `.grid` - default gap |
| **Compact Grid** | 16px - 24px | Widget grids, mobile |

### Section Padding

```css
.section {
  /* Top: dynamic header height, Sides: 20px, Bottom: 80px */
  padding: var(--header-height) 20px 80px;
}

/* Responsive */
@media (max-width: 1024px) {
  padding: var(--header-height) 0 60px; /* No horizontal padding */
}

@media (max-width: 768px) {
  padding: var(--header-height) 0 50px;
}
```

### Container Max-Width

| Screen | Max-Width | Container |
|--------|-----------|-----------|
| All | 1400px | `.container` (legacy) |
| All | 1440px | Site max (Tailwind) |

### Margin Utilities

| Class | Value |
|-------|-------|
| `.mb-lg` | 40px |
| `.mb-xl` | 60px |
| `.mt-md` | 20px |
| `.mt-lg` | 30px |

### Header Height (Dynamic)

- **Desktop**: 80px (set in CSS, synced with JS)
- **Mobile**: 60px - 70px (responsive)
- **Used for**: Scroll padding, section positioning, mobile menu offset

---

## BORDERS & CORNERS

### Border Radius Philosophy

**All corners are SHARP (0px border-radius)** - this is a defining characteristic of the military aesthetic.

| Element | Border Radius |
|---------|--------------|
| Cards, panels, buttons | 0px (sharp) |
| Navigation buttons | 0px (sharp) |
| Icons, badges | 0px (sharp) |
| Modals, overlays | 0px (sharp) |
| **Exception**: Toggle slider (minimal curve) | 0px (flat design) |

### Border Styles

#### **Card Borders**
```css
border: 1px solid var(--bd); /* Default: rgba(68, 117, 146, 0.25) */
```

#### **Panel Borders** (Leaderboards, Sections)
```css
border: 1px solid var(--divider-strong); /* rgba(68, 117, 146, 0.7) */
```

#### **Strong Dividers** (Header Separators)
```css
border-bottom: 2px solid rgba(var(--dl-rgb), 0.45); /* Alert red glow */
```

#### **Soft Dividers** (Row Separators)
```css
border-bottom: 1px solid rgba(var(--mg-rgb), 0.25); /* Subtle steel */
```

### Border Thicknesses

| Thickness | Use Case |
|-----------|----------|
| 1px | Standard borders (cards, panels) |
| 2px | Strong dividers (headers, active states) |
| 3px | Header accent borders (top border emphasis) |

---

## BACKGROUNDS & GRADIENTS

### Solid Backgrounds

| Variable | Hex | Context |
|----------|-----|---------|
| `--bg` | `#050a0f` | Main page background |
| `--s1` | `#0e1a22` | Primary panel |
| `--s2` | `#14222a` | Secondary panel |

### Gradient System

#### **Main Page Gradient** (`--grad-main`)
```css
linear-gradient(135deg, 
  rgba(30, 52, 66, 0.95) 0%, 
  rgba(11, 21, 27, 0.98) 100%)
```
**Use**: Hero sections, main content areas

#### **Card Gradient** (`--grad-card`)
```css
linear-gradient(135deg, 
  rgba(20, 36, 47, 0.95) 0%, 
  rgba(12, 21, 27, 0.98) 100%)
```
**Use**: Card backgrounds, leaderboard rows

#### **Dark Gradient** (`--grad-dark`)
```css
linear-gradient(135deg, 
  rgba(14, 28, 36, 0.96) 0%, 
  rgba(7, 15, 20, 0.98) 100%)
```
**Use**: Deep panels, overlay backgrounds

#### **Panel Gradient** (`--grad-panel`)
```css
linear-gradient(135deg, 
  rgba(12, 22, 28, 0.98) 0%, 
  rgba(22, 36, 46, 0.98) 100%)
```
**Use**: Card surfaces, nested panels

#### **Header Gradient** (Leaderboards - Red)
```css
linear-gradient(180deg, 
  rgba(139, 29, 20, 0.95) 0%, 
  rgba(139, 29, 20, 0.78) 100%)
```
**Color**: Deep Soviet red (`--dl`)

#### **Navigation Gradient**
```css
linear-gradient(180deg, 
  rgba(26, 30, 34, 0.95) 0%, 
  rgba(8, 9, 11, 0.98) 100%)
```
**Use**: Navigation bar background

### Radial Gradients (Hero Only)

**Orange Glow** (top-center)
```css
radial-gradient(ellipse at top, 
  rgba(var(--sw-rgb), 0.12) 0%, 
  transparent 55%)
```

**Steel Glow** (bottom)
```css
radial-gradient(ellipse at bottom, 
  rgba(var(--mg-rgb), 0.15) 0%, 
  transparent 50%)
```

---

## SHADOWS & EFFECTS

### Shadow System

#### **Card Shadow** (`--shadow-card`)
```css
0 12px 24px rgba(5, 10, 15, 0.45)
```
**Use**: Cards, panels on hover, elevated surfaces

#### **Glow Shadow** (`--shadow-glow`)
```css
0 0 8px rgba(68, 117, 146, 0.25)
```
**Use**: Text shadows with glow, ambient effect

#### **Military Panel Shadow** (Leaderboards)
```css
0 12px 30px rgba(4, 9, 14, 0.55), 
inset 0 1px 0 rgba(255, 255, 255, 0.04)
```
**Use**: Panel depth + inner highlight

### Text Shadows

#### **Heading Text Shadow** (H1/H2)
```css
0 2px 10px rgba(0, 0, 0, 0.8), 
0 0 20px rgba(var(--mg-rgb), 0.3)
```

#### **Command Text Shadow** (`.text-command`)
```css
0 0 12px rgba(var(--sw-rgb), 0.45)
```
**Effect**: Orange glow around text

#### **Rank Number Shadow**
```css
0 1px 3px rgba(0, 0, 0, 0.7)
```

### Button Shadows

#### **Primary Button** (`.btn-p`)
```css
/* Normal */
0 6px 20px rgba(var(--sw-rgb), 0.35),
inset 0 1px 0 rgba(255, 255, 255, 0.12)

/* Hover */
0 10px 28px rgba(var(--sw-rgb), 0.45),
0 0 24px rgba(var(--sw-rgb), 0.35),
inset 0 1px 0 rgba(255, 255, 255, 0.18)

/* Active */
0 3px 14px rgba(var(--sw-rgb), 0.45),
inset 0 2px 4px rgba(0, 0, 0, 0.35)
```

#### **Secondary Button** (`.btn-s`)
```css
/* Normal */
0 4px 16px rgba(5, 10, 15, 0.5),
inset 0 1px 0 rgba(255, 255, 255, 0.06)

/* Hover */
0 8px 22px rgba(5, 10, 15, 0.6),
0 0 20px rgba(var(--mg-rgb), 0.25),
inset 0 1px 0 rgba(255, 255, 255, 0.1)
```

#### **Outline Button** (`.btn-outline`)
```css
/* Normal */
0 2px 10px rgba(var(--mg-rgb), 0.25),
inset 0 0 0 1px rgba(var(--mg-rgb), 0.25)

/* Hover */
0 6px 18px rgba(var(--mg-rgb), 0.35),
0 0 20px rgba(var(--sw-rgb), 0.2),
inset 0 0 0 1px rgba(var(--sw-rgb), 0.5)
```

### Glow Effects (for Components)

#### **Massgate Glow** (Tailwind)
```css
0 0 15px rgba(178, 34, 34, 0.4), 
0 0 30px rgba(178, 34, 34, 0.2)
```

#### **Orange Glow** (Tailwind)
```css
0 0 15px rgba(255, 102, 0, 0.4), 
0 0 30px rgba(255, 102, 0, 0.2)
```

---

## BUTTONS

### Button Base Styles (`.btn`)

```css
padding: 14px 28px;
font-family: 'Oswald', sans-serif;
font-size: 14px;
font-weight: 600;
text-transform: uppercase;
letter-spacing: 1px;
border: 2px solid transparent;
border-radius: 0; /* Sharp corners */
cursor: pointer;
display: inline-flex;
align-items: center;
justify-content: center;
gap: 8px;
transition: var(--tr); /* all 0.3s cubic-bezier */
text-shadow: 0 1px 1px rgba(0, 0, 0, 0.35);
```

### Button Variants

#### **Primary** (`.btn-p`) - Orange Command
```css
Background: linear-gradient(180deg, 
  rgba(var(--sw-rgb), 0.9) 0%, 
  rgba(var(--sw-rgb), 0.7) 100%)
Border Color: rgba(var(--sw-rgb), 0.85)
Color: var(--t)
Box Shadow: [primary button shadow above]

/* Hover: brighten and lift */
Background: linear-gradient(180deg, 
  rgba(var(--sw-rgb), 1) 0%, 
  rgba(var(--sw-rgb), 0.75) 100%)
Transform: translateY(-2px)
Box Shadow: [bright primary shadow]

/* Active: darker and pressed */
Transform: translateY(0)
```

#### **Download** (`.btn-download`) - Red Alert
```css
Background: linear-gradient(180deg, 
  rgba(var(--dl-rgb), 0.9) 0%, 
  rgba(var(--dl-rgb), 0.7) 100%)
Border Color: rgba(var(--dl-rgb), 0.8)
Color: #fff

/* Hover: intense glow, scale up */
Box Shadow: 
  0 12px 36px rgba(var(--dl-rgb), 0.6),
  0 0 40px rgba(var(--dl-rgb), 0.5)
Transform: scale(1.05) translateY(-3px)

/* Active: scale down, compress */
Transform: scale(0.96) translateY(2px)
```

#### **Discord** (`.btn-d`) - Blue
```css
Background: linear-gradient(180deg, 
  var(--d) 0%, 
  var(--d-dark) 100%)
Border Color: var(--d-light)
Color: #fff
```

#### **Secondary** (`.btn-s`) - Steel
```css
Background: linear-gradient(180deg, 
  rgba(var(--mg-rgb), 0.35) 0%, 
  rgba(var(--mg-dark-rgb), 0.55) 100%)
Border Color: var(--divider-soft)
Color: var(--t)

/* Hover: orange transition */
Color: var(--sw) /* Changes to orange */
```

#### **Danger** (`.btn-danger`) - Red Alert
```css
Background: linear-gradient(180deg, 
  rgba(var(--dl-light-rgb), 0.95) 0%, 
  rgba(var(--dl-mid-rgb), 0.95) 100%)
Border Color: rgba(var(--dl-light-rgb), 0.8)
Color: #fff
```

#### **Outline** (`.btn-outline`)
```css
Background: transparent
Border Color: var(--divider-strong)
Color: var(--sw)

/* Hover */
Background: linear-gradient(180deg, 
  rgba(var(--mg-rgb), 0.22) 0%, 
  rgba(var(--mg-dark-rgb), 0.22) 100%)
Color: var(--sw)
```

#### **Disabled State** (`.btn:disabled`)
```css
Background: linear-gradient(180deg, 
  rgba(60, 60, 60, 0.5) 0%, 
  rgba(40, 40, 40, 0.5) 100%)
Border Color: rgba(255, 255, 255, 0.1)
Color: rgba(255, 255, 255, 0.4)
Cursor: not-allowed
Box Shadow: none
Transform: none !important
```

### Button Sizes

| Class | Padding | Font Size | Letter Spacing | Use |
|-------|---------|-----------|-----------------|-----|
| **.btn** (default) | 14px 28px | 14px | 1px | Standard buttons |
| **.btn-sm** | 10px 20px | 12px | 0.5px | Small actions |
| **.btn-lg** | 18px 36px | 16px | 1.5px | Primary CTA |
| **.btn-icon** | 12px | N/A | N/A | Icon-only buttons |
| **.btn-icon-sm** | 8px | 12px | N/A | Small icon buttons |

---

## ANIMATIONS & TRANSITIONS

### Transition System

#### **Global Transition** (`--tr`)
```css
all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1)
```
**Used on**: Buttons, cards, nav items, interactive elements

### Keyframe Animations

#### **Pulse** (Status Indicators)
```css
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
```
**Duration**: 2s, ease-in-out, infinite

#### **Sync Pulse** (Server Status)
```css
Same as pulse /* Players online indicator */
```

#### **Glow** (Emphasis)
```css
@keyframes glow {
  0%, 100% { box-shadow: 0 0 5px currentColor; }
  50% {
    box-shadow:
      0 0 20px currentColor,
      0 0 30px currentColor;
  }
}
```

#### **Fade In** (Entry Animation)
```css
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```
**Duration**: 1s, ease-out

#### **Gentle Pulse** (Subtle Emphasis)
```css
@keyframes gentlePulse {
  0%, 100% {
    opacity: 0.8;
    transform: scale(1);
  }
  50% {
    opacity: 1;
    transform: scale(1.1);
  }
}
```

#### **Subtle Glow** (Green Status)
```css
@keyframes subtleGlow {
  0%, 100% { box-shadow: 0 0 4px rgba(124, 179, 66, 0.5); }
  50% { box-shadow: 0 0 8px rgba(124, 179, 66, 0.7); }
}
```

#### **Rank Pulse** (Leaderboard Highlight)
```css
@keyframes rankPulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.05); }
}
```

#### **Score Shine** (Gradient Animation)
```css
@keyframes scoreShine {
  0% { background-position: 200% 50%; }
  100% { background-position: -200% 50%; }
}
```

### Button Tap Animations

#### **Card Tap** (`.cardTap`)
```css
@keyframes cardTap {
  0% { transform: scale(1) translateZ(0); }
  10% { transform: scale(0.98) translateZ(0); }
  90% { transform: scale(0.98) translateZ(0); }
  100% { transform: scale(1) translateZ(0); }
}
```
**Duration**: 0.2s

#### **Button Tap** (`.btnTap`)
```css
@keyframes btnTap {
  0% { transform: scale(1); }
  10% { transform: scale(0.96); }
  90% { transform: scale(0.96); }
  100% { transform: scale(1); }
}
```

#### **Primary Button Tap** (`.btnTapP`)
```css
@keyframes btnTapP {
  0% {
    box-shadow:
      0 8px 25px rgba(var(--mg-rgb), 0.5),
      inset 0 2px 4px rgba(0, 0, 0, 0.2);
  }
  50% { /* hold */ }
  100% {
    box-shadow: 0 4px 15px rgba(var(--mg-rgb), 0.3);
  }
}
```

### Vue Transitions

#### **Mobile Navigation Entry**
```css
.mobile-nav-enter-active,
.mobile-nav-leave-active {
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.mobile-nav-enter-from .mobile-nav-content {
  transform: translateY(-100%);
  opacity: 0;
}
```

#### **Backdrop Fade**
```css
.backdrop-enter-active,
.backdrop-leave-active {
  transition: all 0.3s ease;
}

.backdrop-enter-from,
.backdrop-leave-to {
  opacity: 0;
  backdrop-filter: blur(0px);
}
```

---

## COMPONENT PATTERNS

### Card System (`.card`)

```css
background: var(--grad-card);
border-radius: 0; /* SHARP CORNERS */
border: 1px solid var(--bd);
overflow: hidden;
transition: var(--tr);
position: relative;

/* Hover State */
border-color: rgba(255, 255, 255, 0.15);
transform: translateY(-2px);
box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3);
```

### Panel Container (`.lb-cont`)

Leaderboard panels have enhanced styling:
```css
background: linear-gradient(180deg, 
  rgba(var(--panel-main-rgb), 0.96) 0%, 
  rgba(var(--panel-shadow-rgb), 0.98) 100%);
border: 1px solid var(--divider-strong);
box-shadow: 
  0 12px 30px rgba(4, 9, 14, 0.55),
  inset 0 1px 0 rgba(255, 255, 255, 0.04);
```

### Leaderboard Header (`.lb-hdr`)

```css
padding: 15px 20px;
background: linear-gradient(180deg, 
  rgba(var(--dl-rgb), 0.95) 0%, 
  rgba(var(--dl-rgb), 0.78) 100%);
border-bottom: 3px solid rgba(var(--dl-rgb), 0.85);

h3 {
  font-family: 'Oswald', sans-serif;
  font-size: 1.25rem;
  font-weight: 700;
  color: #fff;
  text-transform: uppercase;
  letter-spacing: 1px;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
}
```

### Tab System (`.tab-btn`, `.tabs`)

**Tab Container** (`.tabs`)
```css
display: flex;
background: linear-gradient(180deg, 
  rgba(var(--graphite-rgb), 0.95) 0%, 
  rgba(var(--graphite-dark-rgb), 0.95) 100%);
border-bottom: 2px solid rgba(var(--graphite-rgb), 0.8);
```

**Tab Button** (`.tab-btn`)
```css
flex: 1;
padding: 12px 16px;
background: linear-gradient(180deg, 
  rgba(var(--graphite-rgb), 0.9) 0%, 
  rgba(var(--graphite-dark-rgb), 0.92) 100%);
border: 1px solid rgba(var(--graphite-dark-rgb), 0.6);
color: var(--t2);
cursor: pointer;
font-family: 'Oswald', sans-serif;
font-weight: 500;
font-size: 0.875rem;
text-transform: uppercase;
letter-spacing: 1px;
transition: var(--tr);

/* Active State */
.tab-btn.active {
  background: linear-gradient(180deg, 
    rgba(var(--sw-rgb), 0.98) 0%, 
    rgba(var(--sw-rgb), 0.8) 100%);
  color: var(--ink);
  font-weight: 600;
  border-color: rgba(var(--sw-rgb), 0.95);
  box-shadow: 0 0 18px rgba(var(--sw-rgb), 0.35);
}
```

### Leaderboard Table (`.lb-table`)

```css
width: 100%;
border-collapse: separate;
border-spacing: 0;
font-family: 'Rajdhani', sans-serif;

/* Headers */
th {
  background: linear-gradient(180deg, 
    rgba(var(--mg-rgb), 0.92) 0%, 
    rgba(var(--mg-dark-rgb), 0.95) 100%);
  color: var(--t);
  padding: 12px;
  font-family: 'Oswald', sans-serif;
  font-weight: 600;
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 1px;
  border: 1px solid rgba(var(--graphite-dark-rgb), 0.6);
  border-bottom: 2px solid rgba(var(--dl-rgb), 0.45);
}

/* Cells */
td {
  padding: 0.75rem;
  color: var(--t);
  border-bottom: 1px solid rgba(var(--mg-rgb), 0.25);
  border-left: 1px solid rgba(var(--mg-rgb), 0.12);
  background: linear-gradient(90deg, 
    rgba(16, 26, 34, 0.75) 0%, 
    rgba(9, 16, 21, 0.85) 100%);
  min-height: 3.5rem;
  height: 3.5rem;

  /* Alternating rows */
  tr:nth-child(even) & {
    background: linear-gradient(90deg, 
      rgba(24, 38, 48, 0.78) 0%, 
      rgba(14, 24, 31, 0.88) 100%);
  }

  /* Hover state */
  tr:hover & {
    background: linear-gradient(90deg, 
      rgba(var(--sw-rgb), 0.18) 0%, 
      rgba(var(--sw-rgb), 0.12) 100%);
    border-color: rgba(var(--sw-rgb), 0.4);
  }
}
```

### Status Indicator (`.status-indicator`)

```css
width: 7px;
height: 7px;
border-radius: 0; /* SQUARE, not rounded */
background: var(--g); /* Green for online */
opacity: 0.9;
transition: opacity 0.3s ease;

/* Animated */
body.players-online & {
  animation: syncPulse 2s ease-in-out infinite;
}
```

### Navigation Links (Desktop)

```css
.desktop-nav a {
  color: var(--t2);
  font-family: 'Oswald', sans-serif;
  font-weight: 500;
  font-size: 14px;
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 0 16px;
  height: 100%;
  background: linear-gradient(180deg, 
    rgba(var(--graphite-rgb), 0.9) 0%, 
    rgba(var(--graphite-dark-rgb), 0.92) 100%);
  border: 1px solid rgba(var(--graphite-dark-rgb), 0.6);
  border-radius: 0;
  transition: var(--tr);

  /* Active */
  &.active {
    background: linear-gradient(180deg, 
      rgba(var(--sw-rgb), 0.98) 0%, 
      rgba(var(--sw-rgb), 0.8) 100%);
    color: var(--ink);
    border-color: rgba(var(--sw-rgb), 0.95);
    transform: scale(1.02) translateY(-2px);
    box-shadow:
      0 4px 20px rgba(0, 0, 0, 0.3),
      0 0 32px rgba(var(--sw-rgb), 0.5);
  }

  /* Bottom border indicator */
  &::before {
    content: '';
    position: absolute;
    bottom: -3px;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(90deg, 
      rgba(var(--sw-rgb), 0.9) 0%, 
      rgba(var(--sw-rgb), 0.6) 100%);
    opacity: 0;
    transition: var(--tr);
  }

  &:hover::before,
  &.active::before {
    opacity: 1;
  }
}
```

### Hero Section (`.hero`)

```css
min-height: 100vh;
background:
  radial-gradient(ellipse at top, 
    rgba(var(--sw-rgb), 0.12) 0%, 
    transparent 55%),
  radial-gradient(ellipse at bottom, 
    rgba(var(--mg-rgb), 0.15) 0%, 
    transparent 50%),
  linear-gradient(180deg, 
    rgba(var(--bg-rgb), 0.96) 0%, 
    rgba(var(--graphite-dark-rgb), 0.98) 100%);
padding-top: var(--header-height);
display: flex;
align-items: center;
```

### Dropdown Menu (`.dropdown-menu`)

```css
position: absolute;
top: 100%;
left: 0;
min-width: 200px;
background: linear-gradient(180deg, 
  rgba(var(--night-rgb), 0.98) 0%, 
  rgba(var(--night-dark-rgb), 0.98) 100%);
border: 1px solid rgba(var(--sw-rgb), 0.85);
border-top: 2px solid rgba(var(--sw-rgb), 0.9);
box-shadow:
  0 8px 32px rgba(0, 0, 0, 0.9),
  0 0 24px rgba(var(--sw-rgb), 0.3);
```

### Community Card (`.com-card`)

```css
background: linear-gradient(180deg,
  rgba(var(--panel-main-rgb), 0.96) 0%,
  rgba(var(--panel-dark-rgb), 0.98) 100%);
border: 1px solid var(--divider-strong);
box-shadow:
  0 12px 28px rgba(4, 9, 14, 0.55),
  inset 0 1px 0 rgba(255, 255, 255, 0.05);
padding: 32px 24px;
cursor: pointer;
text-align: center;

/* Platform-specific borders */
&.discord {
  border: 2px solid var(--d);
  border-top: 3px solid var(--d);
  box-shadow: 0 0 20px rgba(var(--d-light-rgb), 0.2);
}

&.youtube {
  border: 2px solid var(--brand-youtube-peak);
  border-top: 3px solid var(--brand-youtube-peak);
  box-shadow: 0 0 20px rgba(var(--brand-youtube-peak-rgb), 0.2);
}

&.twitch {
  border: 2px solid var(--brand-twitch);
  border-top: 3px solid var(--brand-twitch);
  box-shadow: 0 0 20px rgba(var(--brand-twitch-rgb), 0.2);
}

/* Hover: all transition to orange */
&:hover {
  border-color: rgba(var(--sw-rgb), 0.75) !important;
  border-top-color: rgba(var(--sw-rgb), 0.75) !important;
  box-shadow:
    0 0 30px rgba(var(--sw-rgb), 0.28) !important,
    inset 0 1px 0 rgba(255, 255, 255, 0.12) !important;
  transform: translateY(-2px);
}
```

### Widget Cards (`.widget`)

```css
background: linear-gradient(180deg,
  rgba(var(--panel-main-rgb), 0.96) 0%,
  rgba(var(--panel-dark-rgb), 0.98) 100%);
border: 1px solid var(--divider-strong);
overflow: hidden;
box-shadow: 0 12px 28px rgba(4, 9, 14, 0.55);
transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
display: flex;
flex-direction: column;
min-height: 280px;

&:hover {
  border-color: rgba(var(--sw-rgb), 0.75);
  box-shadow: 0 0 30px rgba(var(--sw-rgb), 0.32);
  transform: translateY(-4px);
}

&.widget-primary {
  background: linear-gradient(180deg,
    rgba(var(--sw-rgb), 0.15) 0%,
    rgba(var(--panel-dark-rgb), 0.98) 100%);
  border-color: rgba(var(--sw-rgb), 0.5);

  &:hover {
    border-color: var(--sw);
    box-shadow: 0 0 40px rgba(var(--sw-rgb), 0.5);
  }
}
```

### Toggle Switch (`.toggle`)

```css
display: inline-flex;
align-items: center;
gap: 10px;
cursor: pointer;

/* Slider Track */
.slider {
  position: relative;
  width: 44px;
  height: 24px;
  background: rgba(var(--t-rgb), 0.12);
  border: 1px solid rgba(var(--t-rgb), 0.2);
  border-radius: 0;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.3);
  transition: var(--tr);
}

/* Slider Thumb */
.slider::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  border-radius: 0;
  background: color-mix(in srgb, var(--t2) 50%, var(--t3) 50%);
  transition: var(--tr);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
}

/* Checked State */
input:checked + .slider {
  background: linear-gradient(135deg, var(--mg) 0%, var(--mg-dark) 100%);
  border-color: rgba(var(--g-rgb), 0.5);
}

input:checked + .slider::after {
  left: 22px;
  background: var(--t);
}
```

---

## RESPONSIVE DESIGN

### Breakpoints (Current Tailwind)

| Name | Width | Use Case |
|------|-------|----------|
| **xs** | 375px | Small phones (custom) |
| **sm** | 640px | Phones |
| **md** | 768px | Tablets |
| **lg** | 1024px | **NAV SWITCH POINT** (desktop nav → mobile) |
| **xl** | 1280px | Large desktop (site max-width 1440px) |

### Header Height Responsive

| Screen | Height | Note |
|--------|--------|------|
| Mobile portrait | 60px | Default mobile |
| Mobile landscape | 65px | Intermediate |
| Tablet | 70px | iPad landscape |
| Desktop | 80px | Full height |

### Navigation Breakpoints

```css
/* Desktop: full horizontal nav */
@media (min-width: 1024px) {
  .desktop-nav { display: flex; }
  .mob-menu { display: none; }
}

/* Mobile: full-screen burger menu */
@media (max-width: 1023px) {
  .desktop-nav { display: none; }
  .mob-menu { display: flex; }
  .mobile-nav { /* Full-screen overlay */ }
}
```

### Responsive Padding

```css
/* Desktop */
.section { padding: var(--header-height) 20px 80px; }

/* Tablet */
@media (max-width: 1024px) {
  padding: var(--header-height) 0 60px; /* Remove horizontal padding */
}

/* Mobile */
@media (max-width: 768px) {
  padding: var(--header-height) 0 50px;
}

/* Small Mobile */
@media (max-width: 480px) {
  padding: var(--header-height) 0 40px;
}
```

### Grid Responsive

```css
/* Desktop: 3 columns */
.widget-grid {
  grid-template-columns: repeat(3, 1fr);
  gap: 24px;
}

/* 2 columns */
@media (max-width: 1100px) {
  grid-template-columns: repeat(2, 1fr);
}

/* Mobile: 1 column */
@media (max-width: 768px) {
  grid-template-columns: 1fr;
  gap: 16px;
}
```

---

## ACCESSIBILITY FEATURES

### Motion Preferences

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

### Focus States

Navigation links have keyboard focus support:
```css
@media (hover: none) and (pointer: coarse) {
  .mob-menu:focus {
    outline: none;
    color: var(--ink);
  }
}
```

### Touch-Friendly Targets

```css
@media (max-width: 768px) {
  .tab-btn, .btn, .mob-menu {
    min-height: 44px; /* iOS guideline */
    min-width: 44px;
  }
}
```

---

## KEY DESIGN PRINCIPLES

1. **Military Aesthetic**: Sharp angles (0px border-radius), bold uppercase typography, structured layout
2. **Dark First**: All backgrounds very dark to reduce eye strain
3. **Steel & Orange**: Primary colors are graphite/steel tones with massgate orange accents
4. **Hierarchy Through Color**: Orange indicates interactive/important, red indicates critical, gray indicates secondary
5. **Strong Shadows**: Depth is created through shadows and gradients, not borders
6. **Consistent Transitions**: All interactive elements use the same cubic-bezier timing
7. **Responsive Mobile**: Header height and padding adjust dynamically; nav switches at 1024px
8. **Performance**: Animations use hardware-accelerated transforms, CSS custom properties for theming

---

## FILE LOCATIONS

### Legacy CSS System
- `/home/micon/dev/wicgate/_archive/legacy/src/assets/styles/modules/variables.css` — All color/spacing vars
- `/home/micon/dev/wicgate/_archive/legacy/src/assets/styles/modules/typography.css` — Font system
- `/home/micon/dev/wicgate/_archive/legacy/src/assets/styles/modules/layout.css` — Grid/spacing
- `/home/micon/dev/wicgate/_archive/legacy/src/assets/styles/modules/buttons.css` — Button variants
- `/home/micon/dev/wicgate/_archive/legacy/src/assets/styles/modules/animations.css` — Keyframes
- `/home/micon/dev/wicgate/_archive/legacy/src/assets/styles/modules/components/*.css` — Component-specific

### Current Nuxt System
- `/home/micon/dev/wicgate/website/tailwind.config.ts` — All design tokens (colors, fonts, spacing)
- `/home/micon/dev/wicgate/website/app/assets/styles/tailwind.css` — Font definitions, base layer
- `/home/micon/dev/wicgate/website/nuxt.config.ts` — CSS configuration, critical styles

---

This comprehensive style guide captures every design detail needed to replicate the WICGATE visual system exactly in another application. The color values are precise hex codes, all spacing is in pixels, and every animation timing is documented.