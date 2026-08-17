---
name: monorepo-patterns
description: "Monorepo architecture for full-stack apps — Turborepo, Nx, shared packages, workspace tooling, CI optimization."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [monorepo, turborepo, nx, workspace, shared-packages, npm-yarn-pnpm]
    related_skills: [project-scaffolding, github-actions-templates, docker-expert]
---

# Monorepo Patterns

## Why Monorepo?

- Share code between frontend and backend
- Atomic commits across packages
- Unified tooling and configuration
- Easy refactoring across the codebase
- Single source of truth for types

**Tradeoffs**: larger repo, potential for slower tooling, need strict boundaries.

---

## Project Structure

```
myapp/
├── apps/
│   ├── web/            # Next.js / Vite frontend
│   ├── api/            # Node.js backend (Express/FastAPI)
│   └── docs/           # Documentation site
├── packages/
│   ├── ui/             # Shared React components
│   ├── config/         # Shared configs (ESLint, TypeScript, Tailwind)
│   ├── tsconfig/       # Base tsconfigs
│   ├── eslint-config/  # Shared ESLint rules
│   ├── tailwind-config/# Shared Tailwind config
│   ├── utils/          # Shared utility functions
│   └── types/          # Shared TypeScript types
├── package.json
├── turbo.json
├── .gitignore
└── package-manager-workspace.yaml
```

---

## Turborepo Setup

```bash
npm install -g turbo
npm install turbo --save-dev -w root
```

```json
// package.json (root)
{
  "name": "myapp",
  "private": true,
  "workspaces": ["apps/*", "packages/*"],
  "scripts": {
    "dev": "turbo dev",
    "build": "turbo build",
    "test": "turbo test",
    "lint": "turbo lint",
    "clean": "turbo clean && rm -rf node_modules"
  },
  "devDependencies": {
    "turbo": "^2.0.0"
  }
}
```

```json
// turbo.json
{
  "$schema": "https://turbo.build/schema.json",
  "pipeline": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": [".next/**", "dist/**", ".turbo/**"]
    },
    "dev": {
      "cache": false,
      "persistent": true
    },
    "test": {
      "dependsOn": ["build"],
      "outputs": ["coverage/**"],
      "inputs": ["src/**/*.tsx", "src/**/*.ts", "test/**/*.ts"]
    },
    "lint": {
      "outputs": []
    },
    "clean": {
      "cache": false
    }
  }
}
```

---

## Shared Configs

```typescript
// packages/tsconfig/base.json
{
  "$schema": "https://json.schemastore.org/tsconfig",
  "compilerOptions": {
    "target": "ES2020",
    "lib": ["ES2020"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "allowJs": true,
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "isolatedModules": true,
    "declaration": true,
    "declarationMap": true
  }
}

// packages/tsconfig/nextjs.json
{
  "extends": "./base.json",
  "compilerOptions": {
    "jsx": "preserve",
    "plugins": [{ "name": "next" }]
  }
}
```

```json
// apps/web/tsconfig.json
{
  "extends": "@myapp/tsconfig/nextjs.json",
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

---

## Shared UI Package

```typescript
// packages/ui/package.json
{
  "name": "@myapp/ui",
  "version": "0.0.0",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": "./dist/index.js",
    "./button": "./dist/button.js",
    "./card": "./dist/card.js"
  },
  "scripts": {
    "build": "tsup",
    "typecheck": "tsup"
  }
}
```

```typescript
// packages/ui/index.ts
export { Button, type ButtonProps } from './button';
export { Card, type CardProps } from './card';
export { Input, type InputProps } from './input';
```

```typescript
// packages/ui/button.tsx
import * as React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost';
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = 'primary', className = '', ...props }, ref) => (
    <button
      ref={ref}
      className={`btn btn-${variant} ${className}`}
      {...props}
    />
  )
);
Button.displayName = 'Button';
```

---

## CI/CD with Turborepo

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: pnpm/action-setup@v3
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'

      - run: pnpm install --frozen-lockfile

      - run: pnpm turbo lint test build
        env:
          TURBO_TOKEN: ${{ secrets.TURBO_TOKEN }}
          TURBO_TEAM: ${{ vars.TURBO_TEAM }}

      - uses: codecov/codecov-action@v4

  deploy:
    needs: ci
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v3
      - uses: actions/setup-node@v4
      - run: pnpm install --frozen-lockfile
      - run: pnpm turbo deploy --filter=api --filter=web
        env:
          VERCEL_TOKEN: ${{ secrets.VERCEL_TOKEN }}
```

---

## Docker in Monorepo

```dockerfile
# From root
FROM node:20-alpine AS builder
WORKDIR /app

# Copy workspace manifests first (for layer caching)
COPY package*.json pnpm-lock.yaml ./
COPY packages/*/package.json packages/
COPY apps/*/package.json apps/

RUN npm install -g pnpm && pnpm install --frozen-lockfile

COPY . .
RUN pnpm turbo build --filter=api

# Production image for the API
FROM node:20-alpine AS production
WORKDIR /app
COPY --from=builder --chown=nodeapp:nodeapp /app/apps/api/dist ./dist
COPY --from=builder --chown=nodeapp:nodeapp /app/node_modules ./node_modules
COPY --from=builder --chown=nodeapp:nodeapp /app/packages ./packages
CMD ["node", "dist/server.js"]
```

---

## pnpm Workspace Config

```yaml
# pnpm-workspace.yaml
packages:
  - 'apps/*'
  - 'packages/*'
```

```bash
# Install all deps
pnpm install

# Add to specific workspace
pnpm add zod --filter=@myapp/ui
pnpm add express --filter=api

# Build all dependents
pnpm turbo build --filter=@myapp/ui...  # Build ui AND everything that depends on it

# Run in single package
pnpm --filter=api dev
```

---

## Best Practices

1. **Shared types package** — single source of truth for API types, shared between frontend and backend
2. **Strict module boundaries** — use ESLint rules to prevent `apps/web` importing from `apps/api`
3. **Version bumps** — use `changesets` for managing versions across packages
4. **CI caching** — leverage Turborepo's remote cache (vercel.com/turborepo) for faster CI
5. **Small packages** — prefer many small packages over one giant `shared` package
6. **Internal packages** — use `publishConfig: { access: 'restricted' }` for internal packages
7. **Lockfile** — use `pnpm` for best monorepo support (workspace protocol, isolated installs)
