---
name: Tailwind CSS Design System
description: Tailwind CSS v4 patterns and design system creation for building consistent, scalable UIs
triggers:
  - tailwind
  - tailwind css
  - tailwind v4
  - design system
  - component library
version: 1.0.0
tags:
  - css
  - tailwind
  - design-system
  - frontend
  - styling
---

# Tailwind CSS v4 Design System

## Core Concepts

### Utility-First CSS
Tailwind provides low-level utility classes for rapid UI development without leaving your HTML.

```html
<button class="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded-lg font-medium transition-colors">
  Click me
</button>
```

### Custom Themes
```js
// tailwind.config.js (v3) or @theme in CSS (v4)
@theme {
  --color-primary: #3b82f6;
  --color-primary-dark: #2563eb;
  --color-secondary: #8b5cf6;
  --radius-lg: 1rem;
  --font-display: 'Inter', sans-serif;
  --font-mono: 'JetBrains Mono', monospace;
}
```

## Design Token System

### Color Palette
```css
@theme {
  /* Primary scale */
  --color-primary-50: #eff6ff;
  --color-primary-100: #dbeafe;
  --color-primary-200: #bfdbfe;
  --color-primary-300: #93c5fd;
  --color-primary-400: #60a5fa;
  --color-primary-500: #3b82f6;
  --color-primary-600: #2563eb;
  --color-primary-700: #1d4ed8;
  --color-primary-800: #1e40af;
  --color-primary-900: #1e3a8a;
  --color-primary-950: #172554;

  /* Semantic colors */
  --color-success: #22c55e;
  --color-warning: #f59e0b;
  --color-error: #ef4444;
  --color-info: #06b6d4;
}
```

### Typography Scale
```css
@theme {
  --font-sans: 'Inter', ui-sans-serif, system-ui, sans-serif;
  --font-display: 'Plus Jakarta Sans', ui-sans-serif, sans-serif;
  --font-mono: 'JetBrains Mono', ui-monospace, monospace;

  --text-xs: 0.75rem;    /* 12px */
  --text-sm: 0.875rem;   /* 14px */
  --text-base: 1rem;     /* 16px */
  --text-lg: 1.125rem;   /* 18px */
  --text-xl: 1.25rem;    /* 20px */
  --text-2xl: 1.5rem;    /* 24px */
  --text-3xl: 1.875rem;  /* 30px */
  --text-4xl: 2.25rem;   /* 36px */
}
```

### Spacing System
```css
@theme {
  --spacing-xs: 0.25rem;   /* 4px */
  --spacing-sm: 0.5rem;    /* 8px */
  --spacing-md: 1rem;      /* 16px */
  --spacing-lg: 1.5rem;   /* 24px */
  --spacing-xl: 2rem;      /* 32px */
  --spacing-2xl: 3rem;     /* 48px */
}
```

## Component Patterns

### Button Component
```css
@utility btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.625rem 1.25rem;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 0.5rem;
  transition: all 150ms ease;
  cursor: pointer;
}

@utility btn-primary {
  background-color: var(--color-primary);
  color: white;
}

@utility btn-primary:hover {
  background-color: var(--color-primary-dark);
}

@utility btn-secondary {
  background-color: transparent;
  border: 1px solid var(--color-gray-300);
}

@utility btn-sm {
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
}

@utility btn-lg {
  padding: 0.875rem 1.75rem;
  font-size: 1rem;
}
```

### Card Component
```css
@utility card {
  background-color: white;
  border-radius: 0.75rem;
  box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
  overflow: hidden;
}

@utility card-header {
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--color-gray-100);
}

@utility card-body {
  padding: 1.5rem;
}

@utility card-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--color-gray-100);
  background-color: var(--color-gray-50);
}
```

### Input Component
```css
@utility input {
  display: block;
  width: 100%;
  padding: 0.625rem 1rem;
  font-size: 0.875rem;
  border: 1px solid var(--color-gray-300);
  border-radius: 0.5rem;
  background-color: white;
  transition: border-color 150ms, box-shadow 150ms;
}

@utility input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-100);
}

@utility input-error {
  border-color: var(--color-error);
}

@utility input-disabled {
  background-color: var(--color-gray-100);
  cursor: not-allowed;
  opacity: 0.7;
}
```

## Layout Patterns

### Responsive Grid
```html
<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
  <div class="card">...</div>
  <div class="card">...</div>
  <div class="card">...</div>
</div>
```

### Container
```css
@utility container-sm {
  max-width: 640px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1rem;
  padding-right: 1rem;
}

@utility container-md {
  max-width: 768px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 1.5rem;
  padding-right: 1.5rem;
}

@utility container-lg {
  max-width: 1024px;
  margin-left: auto;
  margin-right: auto;
  padding-left: 2rem;
  padding-right: 2rem;
}
```

### Sidebar Layout
```css
@utility layout-sidebar {
  display: grid;
  grid-template-columns: 280px 1fr;
  min-height: 100vh;
}

@utility sidebar-nav {
  padding: 1.5rem;
  background-color: var(--color-gray-50);
  border-right: 1px solid var(--color-gray-200);
}

@media (max-width: 768px) {
  @utility layout-sidebar {
    grid-template-columns: 1fr;
  }
}
```

## Animation & Motion

### Transitions
```css
@utility transition-base {
  transition-property: all;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 150ms;
}

@utility transition-slow {
  transition-duration: 300ms;
}

@utility hover-lift {
  transition-property: transform, box-shadow;
}

@utility hover-lift:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
}
```

### Keyframes
```css
@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slide-up {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes pulse-soft {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

@utility animate-fade-in {
  animation: fade-in 300ms ease-out;
}

@utility animate-slide-up {
  animation: slide-up 300ms ease-out;
}

@utility animate-pulse-soft {
  animation: pulse-soft 2s ease-in-out infinite;
}
```

## Dark Mode

### Class-based Dark Mode
```js
// tailwind.config.js
module.exports = {
  darkMode: 'class',
  // ...
}
```

```css
@theme {
  --color-bg-dark: #0f172a;
  --color-surface-dark: #1e293b;
  --color-text-dark: #f1f5f9;
}

@utility bg-primary {
  background-color: white;
}

@utility dark-bg-primary {
  background-color: var(--color-bg-dark);
}
```

```html
<!-- Usage -->
<div class="bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100">
  <button class="bg-blue-500 hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700">
    Theme Toggle
  </button>
</div>
```

## State Variants

### Custom States
```css
@utility btn {
  /* Base styles */
}

@utility btn[data-loading="true"] {
  opacity: 0.7;
  cursor: wait;
}

@utility input[data-error="true"] {
  border-color: var(--color-error);
  border-width: 2px;
}
```

### Group Hover
```html
<div class="group relative">
  <img src="..." class="group-hover:opacity-75 transition-opacity" />
  <div class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100">
    <span class="text-white">View Details</span>
  </div>
</div>
```

### Peer States
```html
<label class="peer">
  <input type="checkbox" class="peer-checked:bg-blue-500" />
  <span class="peer-checked:text-blue-600">Accept terms</span>
</label>
```

## Tailwind v4 Features

### @apply with Custom Classes
```css
@utility card {
  background: white;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
}

.my-custom-card {
  @apply card p-6;
}
```

### CSS-first Configuration
```css
/* In your main CSS file */
@import "tailwindcss";

@theme {
  --color-brand: #6366f1;
  --radius-xl: 1.25rem;
  --animate-bounce-slow: bounce 2s infinite;
}

.btn {
  background-color: var(--color-brand);
  border-radius: var(--radius-xl);
  animation: var(--animate-bounce-slow);
}
```

### Container Queries (v4)
```html
<div class="@container">
  <div class="@md:flex @lg:grid-cols-3">
    <!-- Responsive to container, not viewport -->
  </div>
</div>
```

## Best Practices

1. **Extract Components**: Repeated class patterns should become components
2. **Design Tokens**: Define semantic colors (primary, danger) over raw hex
3. **Responsive Design**: Mobile-first with `sm:`, `md:`, `lg:`, `xl:` breakpoints
4. **Dark Mode**: Test both modes during development
5. **Minification**: Use `@layer` to organize utilities
6. **Arbitrary Values**: Use sparingly; prefer theme extensions
7. **Composing Utilities**: Small utilities combine well; avoid monolithic classes
8. **CSSLayers**: Use `@layer components` for custom component styles to avoid specificity issues
