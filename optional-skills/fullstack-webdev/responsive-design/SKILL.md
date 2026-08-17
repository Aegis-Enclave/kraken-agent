---
name: responsive-design
description: Design and implement responsive, mobile-first web layouts using CSS media queries, flexible grid systems, modern CSS layout (Flexbox, Grid), container queries, and viewport units. Covers responsive typography, images, navigation patterns, and testing strategies across devices. NOT for native mobile apps — use React Native or Flutter for those.
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [css, responsive, mobile-first, media-queries, flexbox, grid, layout, viewport, ux]
    category: fullstack-webdev
---

# Responsive Design

Responsive web design ensures websites adapt gracefully to different screen sizes, from mobile phones to large desktop monitors. The mobile-first approach starts with styles for small screens and progressively enhances for larger viewports using CSS media queries and modern layout techniques.

## When to use this skill

Load this skill when a user wants to:

- Create responsive layouts that work across all device sizes
- Implement mobile-first design with progressive enhancement
- Build flexible grid and component systems
- Handle responsive images, typography, and navigation
- Use modern CSS features like container queries and clamp()
- Debug responsive issues across breakpoints
- Convert fixed-width designs to fluid responsive layouts

## When NOT to use this skill

- User wants native mobile app development (use React Native, Flutter, or Swift/Kotlin)
- User needs responsive email templates (use email-specific frameworks)
- User has complex 3D or game content (use canvas/WebGL responsive patterns)
- User wants print-specific stylesheets only

## Core Concepts

### Mobile-First vs Desktop-First

**Mobile-first** (recommended):
```css
/* Base styles (mobile) */
.container { padding: 1rem; }

/* Larger screens */
@media (min-width: 768px) {
  .container { padding: 2rem; max-width: 1200px; }
}
```

**Desktop-first** (legacy approach):
```css
/* Base styles (desktop) */
.container { padding: 2rem; max-width: 1200px; }

/* Smaller screens */
@media (max-width: 767px) {
  .container { padding: 1rem; }
}
```

### Breakpoints

Standard breakpoint conventions (Tailwind-inspired):

| Breakpoint | Min-width | Typical use |
|------------|-----------|-------------|
| sm | 640px | Large phones |
| md | 768px | Tablets |
| lg | 1024px | Laptops |
| xl | 1280px | Desktops |
| 2xl | 1536px | Large screens |

CSS custom properties for consistency:
```css
:root {
  --breakpoint-sm: 640px;
  --breakpoint-md: 768px;
  --breakpoint-lg: 1024px;
  --breakpoint-xl: 1280px;
  --breakpoint-2xl: 1536px;
}
```

## Layout Techniques

### CSS Grid

**Basic responsive grid:**
```css
.grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1rem;
}

@media (min-width: 768px) {
  .grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (min-width: 1024px) {
  .grid {
    grid-template-columns: repeat(3, 1fr);
  }
}
```

**Auto-fit and auto-fill:**
```css
/* Automatically fits as many columns as possible */
.auto-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1rem;
}
```

**Named grid areas:**
```css
.layout {
  display: grid;
  grid-template-areas:
    "header"
    "main"
    "sidebar"
    "footer";
  grid-template-rows: auto 1fr auto auto;
}

@media (min-width: 768px) {
  .layout {
    grid-template-areas:
      "header header"
      "main sidebar"
      "footer footer";
    grid-template-columns: 1fr 300px;
  }
}

.header { grid-area: header; }
.main { grid-area: main; }
.sidebar { grid-area: sidebar; }
.footer { grid-area: footer; }
```

### Flexbox

**Responsive navigation:**
```css
.nav {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

@media (min-width: 768px) {
  .nav {
    flex-direction: row;
    justify-content: space-between;
  }
}
```

**Sticky footer:**
```css
.page-wrapper {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.main-content {
  flex: 1;
}
```

**Flexible card layout:**
```css
.card-container {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

.card {
  flex: 1 1 300px; /* Grow, shrink, basis */
  max-width: 400px;
}
```

## Container Queries

Container queries allow components to respond to their container's size rather than the viewport — ideal for reusable components.

```css
/* Define a container */
.card-wrapper {
  container-type: inline-size;
  container-name: card;
}

/* Query the container */
@container card (min-width: 400px) {
  .card {
    display: grid;
    grid-template-columns: 1fr 2fr;
  }
}

@container card (max-width: 399px) {
  .card {
    display: flex;
    flex-direction: column;
  }
}
```

**With named sizes:**
```css
.container {
  container-type: inline-size;
  container-name: card-container;
}

@container card-container (width > 600px) {
  .card { padding: 2rem; }
}
```

## Responsive Typography

### Fluid Type with clamp()

```css
/* Fluid font size between 16px and 24px */
.fluid-text {
  font-size: clamp(1rem, 0.875rem + 0.5vw, 1.5rem);
}

/* Headings scale smoothly */
h1 {
  font-size: clamp(2rem, 1rem + 3vw, 4rem);
  line-height: 1.1;
}

h2 {
  font-size: clamp(1.5rem, 1rem + 2vw, 2.5rem);
  line-height: 1.2;
}

/* Paragraph text */
p {
  font-size: clamp(0.875rem, 0.75rem + 0.5vw, 1.125rem);
  max-width: 65ch; /* Optimal reading width */
}
```

### Responsive Type Scale

```css
:root {
  --text-xs: clamp(0.75rem, 0.7rem + 0.25vw, 0.875rem);
  --text-sm: clamp(0.875rem, 0.8rem + 0.35vw, 1rem);
  --text-base: clamp(1rem, 0.9rem + 0.5vw, 1.125rem);
  --text-lg: clamp(1.125rem, 1rem + 0.5vw, 1.25rem);
  --text-xl: clamp(1.25rem, 1rem + 1vw, 1.5rem);
  --text-2xl: clamp(1.5rem, 1rem + 2vw, 2rem);
  --text-3xl: clamp(1.875rem, 1.5rem + 2vw, 3rem);
}
```

## Responsive Images

### srcset and sizes

```html
<img
  src="image-800.jpg"
  srcset="image-400.jpg 400w,
          image-800.jpg 800w,
          image-1200.jpg 1200w,
          image-1600.jpg 1600w"
  sizes="(max-width: 600px) 100vw,
         (max-width: 1200px) 50vw,
         33vw"
  alt="Responsive image"
  loading="lazy"
/>
```

### picture element for art direction

```html
<picture>
  <source
    media="(min-width: 1024px)"
    srcset="hero-desktop.jpg"
  />
  <source
    media="(min-width: 768px)"
    srcset="hero-tablet.jpg"
  />
  <img src="hero-mobile.jpg" alt="Hero image" />
</picture>
```

### CSS background images

```css
.hero {
  background-image: url("hero-mobile.jpg");
  background-size: cover;
  background-position: center;
}

@media (min-width: 768px) {
  .hero {
    background-image: url("hero-tablet.jpg");
  }
}

@media (min-width: 1024px) {
  .hero {
    background-image: url("hero-desktop.jpg");
  }
}
```

### Modern image formats

```html
<picture>
  <source type="image/avif" srcset="image.avif" />
  <source type="image/webp" srcset="image.webp" />
  <img src="image.jpg" alt="Description" />
</picture>
```

## Responsive Navigation Patterns

### Hamburger menu (CSS-only approach)

```css
/* Hide checkbox */
.menu-toggle { display: none; }

/* Hide nav by default on mobile */
.main-nav {
  display: none;
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: white;
}

/* Show when checkbox is checked */
.menu-toggle:checked ~ .main-nav {
  display: block;
}

/* Show inline on desktop */
@media (min-width: 768px) {
  .menu-toggle,
  .menu-icon { display: none; }
  
  .main-nav {
    display: flex;
    position: static;
    flex-direction: row;
    gap: 2rem;
    background: transparent;
  }
}
```

```html
<nav class="main-nav-wrapper">
  <label for="menu-toggle" class="menu-icon">Menu</label>
  <input type="checkbox" id="menu-toggle" class="menu-toggle" />
  <ul class="main-nav">
    <li><a href="/">Home</a></li>
    <li><a href="/about">About</a></li>
    <li><a href="/contact">Contact</a></li>
  </ul>
</nav>
```

### Accessible navigation with JavaScript

```html
<nav class="navbar" id="navbar">
  <button
    class="menu-button"
    aria-expanded="false"
    aria-controls="nav-menu"
    aria-label="Toggle navigation"
  >
    <span class="hamburger"></span>
  </button>
  <ul id="nav-menu" class="nav-menu" hidden>
    <li><a href="/">Home</a></li>
    <li><a href="/about">About</a></li>
  </ul>
</nav>
```

```javascript
const menuButton = document.querySelector('.menu-button')
const navMenu = document.querySelector('#nav-menu')

menuButton.addEventListener('click', () => {
  const isOpen = !navMenu.hidden
  navMenu.hidden = isOpen
  menuButton.setAttribute('aria-expanded', !isOpen)
})
```

## Viewport Units and Fluid Layouts

### Viewport units

| Unit | Description |
|------|-------------|
| vw | 1% of viewport width |
| vh | 1% of viewport height |
| vmin | 1% of smaller viewport dimension |
| vmax | 1% of larger viewport dimension |
| dvh | Dynamic viewport height (mobile browsers) |
| svh | Small viewport height |
| lvh | Large viewport height |

### Using dvh for mobile

```css
/* Handles mobile browser chrome */
.hero {
  height: 100dvh; /* Dynamic viewport height */
  min-height: -webkit-fill-available;
}
```

### Viewport-relative spacing

```css
.section {
  padding: clamp(2rem, 5vw, 4rem) clamp(1rem, 3vw, 2rem);
}

.card {
  padding: calc(1rem + 2vw);
}
```

## Responsive Spacing System

```css
:root {
  --space-1: 0.25rem;   /* 4px */
  --space-2: 0.5rem;    /* 8px */
  --space-3: 0.75rem;   /* 12px */
  --space-4: 1rem;      /* 16px */
  --space-6: 1.5rem;    /* 24px */
  --space-8: 2rem;      /* 32px */
  --space-12: 3rem;     /* 48px */
  --space-16: 4rem;     /* 64px */
}

@media (min-width: 768px) {
  :root {
    --space-1: 0.5rem;
    --space-2: 1rem;
    --space-4: 1.5rem;
    --space-8: 3rem;
    --space-16: 6rem;
  }
}
```

## Testing Responsive Designs

### Browser DevTools

**Chrome/Edge:**
- DevTools > Toggle device toolbar (Ctrl/Cmd + Shift + M)
- Preset devices or custom dimensions
- Throttle network for mobile testing

**Firefox:**
- DevTools > Responsive Design Mode
- Edit screen sizes in settings

### Testing Tools

| Tool | Purpose |
|------|---------|
| BrowserStack | Real device testing |
| LambdaTest | Cross-browser testing |
| Responsinator | Quick viewport previews |
| Polypane | Multiple viewports simultaneously |

### Print stylesheet consideration

```css
@media print {
  .no-print { display: none; }
  .print-only { display: block; }
  
  body {
    font-size: 12pt;
    line-height: 1.5;
    color: black;
  }
  
  a::after {
    content: " (" attr(href) ")";
    font-size: 0.8em;
  }
}
```

## Common Responsive Patterns

### Sidebar + Main Content

```css
.page-layout {
  display: grid;
  grid-template-columns: 1fr;
}

@media (min-width: 1024px) {
  .page-layout {
    grid-template-columns: 250px 1fr;
  }
  
  .sidebar {
    position: sticky;
    top: 2rem;
    height: fit-content;
  }
}
```

### Responsive Card Grid

```css
.card-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1rem;
}

@media (min-width: 480px) {
  .card-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (min-width: 768px) {
  .card-grid {
    grid-template-columns: repeat(3, 1fr);
    gap: 1.5rem;
  }
}

@media (min-width: 1200px) {
  .card-grid {
    grid-template-columns: repeat(4, 1fr);
  }
}
```

### Two-Column Layout (Tablet +)

```css
.two-columns {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

@media (min-width: 768px) {
  .two-columns {
    flex-direction: row;
    gap: 2rem;
  }
  
  .two-columns > * {
    flex: 1;
  }
}
```

### Hide/Show Based on Screen Size

```css
/* Hide on mobile, show on desktop */
.desktop-only { display: none; }
@media (min-width: 768px) {
  .desktop-only { display: block; }
}

/* Hide on desktop, show on mobile */
.mobile-only { display: block; }
@media (min-width: 768px) {
  .mobile-only { display: none; }
}

/* Show only on specific sizes */
.tablet-only {
  display: none;
}
@media (min-width: 768px) and (max-width: 1023px) {
  .tablet-only { display: block; }
}
```

## CSS Custom Properties for Responsive Design

```css
:root {
  /* Container widths */
  --container-sm: 640px;
  --container-md: 768px;
  --container-lg: 1024px;
  --container-xl: 1280px;
  
  /* Grid columns */
  --grid-columns: 4;
}

@media (min-width: 768px) {
  :root {
    --grid-columns: 8;
  }
}

@media (min-width: 1024px) {
  :root {
    --grid-columns: 12;
  }
}

.container {
  width: 100%;
  max-width: var(--container-md);
  margin: 0 auto;
  padding: 0 var(--space-4);
}

@media (min-width: 768px) {
  .container { max-width: var(--container-lg); }
}

@media (min-width: 1024px) {
  .container { max-width: var(--container-xl); padding: 0 var(--space-6); }
}
```

## Performance Considerations

### Critical CSS

```html
<style>
  /* Inline critical above-the-fold CSS */
  .hero { min-height: 100vh; }
</style>
<link rel="stylesheet" href="styles.css" media="print" onload="this.media='all'" />
```

### Lazy loading images

```html
<img src="placeholder.jpg" data-src="actual-image.jpg" class="lazy" alt="..." />
```

```javascript
// Intersection Observer for lazy loading
const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      const img = entry.target
      img.src = img.dataset.src
      img.classList.remove('lazy')
      observer.unobserve(img)
    }
  })
})
```

## Reference

- MDN Responsive Design: https://developer.mozilla.org/en-US/docs/Learn/CSS/CSS_layout/Responsive_Design
- CSS Tricks - Responsive Design: https://css-tricks.com/snippets/css/a-guide-to-media-queries/
- Container Queries: https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_container_queries
- Modern CSS Solutions: https://moderncss.dev/
- Fluid Type Scale Calculator: https://utopia.fyi/
