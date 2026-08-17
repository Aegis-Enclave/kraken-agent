---
name: project-scaffolding
description: Scaffold production-ready full-stack web projects with modern tooling, monorepo support, CI/CD pipelines, and infrastructure-as-code. Use when initializing a new project, adding features to an existing codebase, setting up testing infrastructure, or configuring deployment pipelines.
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [scaffold, project, fullstack, monorepo, turbo, nx, docker, ci-cd, github-actions, terraform, deployment]
    homepage: https://github.com
    related_skills: [typescript-mastery]
prerequisites:
  node: ">=18.0.0"
  package_managers: [npm, pnpm, yarn]
---

# Project Scaffolding

Rapidly scaffold production-ready full-stack web projects with battle-tested configurations, modern tooling, and enterprise-grade infrastructure.

## When to Use

- Initialize a new full-stack project (frontend + backend)
- Add packages/features to an existing monorepo
- Set up testing infrastructure (unit, integration, E2E)
- Configure CI/CD pipelines for any major platform
- Create Docker/container configurations
- Set up infrastructure-as-code (IaC)
- Configure environment-based secrets management

## Supported Stack Templates

### Frontend Frameworks
- **Next.js** (App Router) — React full-stack with SSR/SSG
- **Astro** — Content-focused with island architecture
- **SvelteKit** — Svelte full-stack
- **Remix** — React framework with web standards
- **Vite SPA** — Vanilla TS/React/Vue SPA

### Backend Frameworks
- **Express/Fastify** — Node.js REST/GraphQL API
- **NestJS** — TypeScript-first Node.js framework
- **tRPC** — End-to-end typesafe APIs
- **Hono** — Lightweight, edge-ready

### Databases
- **PostgreSQL** (via Prisma, Drizzle, or raw)
- **MongoDB** (via Mongoose)
- **SQLite** (for local dev)

### Monorepo Tools
- **Turborepo** (Vercel) — Fast, minimal config
- **Nx** — Powerful, extensible

## Quick Start: Monorepo with Turbo

### 1. Create the project

```bash
# Initialize with pnpm workspaces
mkdir my-fullstack-app && cd my-fullstack-app
pnpm init -y

# Install turbo
pnpm add -D turbo

# Create workspace structure
mkdir -p apps/{web,api} packages/{ui,config,tsconfig}
```

### 2. Configure workspace

```json
// pnpm-workspace.yaml
packages:
  - "apps/*"
  - "packages/*"
```

```json
// turbo.json
{
  "$schema": "https://turbo.build/schema.json",
  "globalDependencies": [".env"],
  "pipeline": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": [".next/**", "dist/**"]
    },
    "dev": {
      "cache": false,
      "persistent": true
    },
    "lint": {},
    "test": {
      "dependsOn": ["build"]
    }
  }
}
```

### 3. Create apps

**Frontend (Next.js App Router):**

```bash
cd apps/web
pnpm create next-app@latest . --typescript --tailwind --eslint --app --src-dir --import-alias "@/*" --no-git
```

**Backend (Express + tRPC):**

```bash
cd apps/api
pnpm init
pnpm add express cors helmet zod
pnpm add -D typescript @types/express @types/cors ts-node nodemon
```

### 4. Share TypeScript config

```json
// packages/tsconfig/base.json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "declarationMap": true
  }
}
```

```json
// packages/tsconfig/nextjs.json
{
  "extends": "./base.json",
  "compilerOptions": {
    "jsx": "preserve",
    "plugins": [{ "name": "next" }]
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

## Docker Configuration

### Multi-stage Dockerfile for Next.js

```dockerfile
# apps/web/Dockerfile
FROM node:20-alpine AS base

# Install dependencies only when needed
FROM base AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY packages ./packages
COPY apps/web/package.json ./apps/web/
RUN npm install -g pnpm@9 && pnpm install --frozen-lockfile

# Rebuild the source code
FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN pnpm build

# Production image
FROM base AS runner
WORKDIR /app
ENV NODE_ENV=production
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs
COPY --from=builder --chown=nextjs:nodejs /app/apps/web/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/apps/web/.next/static ./.next/static
USER nextjs
EXPOSE 3000
ENV PORT=3000
CMD ["node", "server.js"]
```

### Docker Compose for local dev

```yaml
# docker-compose.yml
version: "3.9"
services:
  web:
    build:
      context: .
      dockerfile: apps/web/Dockerfile.dev
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@db:5432/myapp
      - NEXT_PUBLIC_API_URL=http://api:4000
    depends_on:
      - db
      - redis
    volumes:
      - ./apps/web:/app
    command: pnpm dev

  api:
    build:
      context: .
      dockerfile: apps/api/Dockerfile.dev
    ports:
      - "4000:4000"
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@db:5432/myapp
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
    volumes:
      - ./apps/api:/app
    command: pnpm dev

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: myapp
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

## CI/CD: GitHub Actions

### Full-stack test + deploy pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  TURBO_TOKEN: ${{ secrets.TURBO_TOKEN }}
  TURBO_TEAM: ${{ vars.TURBO_TEAM }}

jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm lint

  typecheck:
    name: Type Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm typecheck

  test:
    name: Test
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - name: Run tests
        run: pnpm test
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/test?schema=public

  build:
    name: Build
    runs-on: ubuntu-latest
    needs: [lint, typecheck, test]
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm build
      - uses: actions/upload-artifact@v4
        with:
          name: dist
          path: .
          retention-days: 7

  deploy:
    name: Deploy
    runs-on: ubuntu-latest
    needs: [build]
    if: github.ref == 'refs/heads/main'
    environment: production
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - name: Download dist
        uses: actions/download-artifact@v4
        with:
          name: dist
      - name: Deploy
        run: pnpm deploy:prod
        env:
          FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}
```

## Testing Infrastructure

### Vitest setup (recommended)

```bash
pnpm add -D vitest @vitest/coverage-v8 @vitest/ui jsdom
```

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
  plugins: [react(), tsconfigPaths()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'src/test/']
    }
  }
})
```

```typescript
// src/test/setup.ts
import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Mock environment variables
vi.stubEnv('NEXT_PUBLIC_API_URL', 'http://localhost:4000')

// Mock fetch
global.fetch = vi.fn()
```

### Playwright E2E setup

```bash
pnpm add -D @playwright/test
pnpm exec playwright install --with-deps chromium
```

```typescript
// e2e/example.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Auth flow', () => {
  test('should login with valid credentials', async ({ page }) => {
    await page.goto('/login')
    await page.getByLabel('Email').fill('user@example.com')
    await page.getByLabel('Password').fill('password123')
    await page.getByRole('button', { name: 'Sign in' }).click()
    await expect(page).toHaveURL('/dashboard')
    await expect(page.getByText('Welcome back')).toBeVisible()
  })
})
```

## Environment & Secrets Management

### .env.example

```bash
# apps/web/.env.example
# Copy to .env.local for local development

# Public (safe to commit, prefixed with NEXT_PUBLIC_)
NEXT_PUBLIC_API_URL=http://localhost:4000
NEXT_PUBLIC_APP_URL=http://localhost:3000

# Server-only (never commit)
DATABASE_URL=postgresql://user:password@localhost:5432/mydb
NEXTAUTH_SECRET=your-secret-here
NEXTAUTH_URL=http://localhost:3000
```

### GitHub Actions secrets

Add via: Repository Settings → Secrets and variables → Actions

```
DATABASE_URL
NEXTAUTH_SECRET
TURBO_TOKEN
TURBO_TEAM
FLY_API_TOKEN
```

## Infrastructure as Code (Terraform)

### AWS ECS deployment

```hcl
# infra/main.tf
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = { source = "hashicorp/aws" }
  }
}

provider "aws" {
  region = var.aws_region
}

resource "aws_ecs_cluster" "main" {
  name = "${var.project}-cluster"
}

resource "aws_ecs_service" "web" {
  name            = "${var.project}-web"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.web.arn
  desired_count   = var.desired_count
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = var.private_subnet_ids
    security_groups  = [aws_security_group.ecs.id]
    assign_public_ip = false
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.web.arn
    container_name   = "web"
    container_port   = 3000
  }
}

resource "aws_lb" "main" {
  name               = "${var.project}-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = var.public_subnet_ids
}

resource "aws_lb_target_group" "web" {
  name     = "${var.project}-web-tg"
  port     = 3000
  protocol = "HTTP"
  vpc_id   = var.vpc_id
}

resource "aws_security_group" "ecs" {
  name = "${var.project}-ecs-sg"
  ingress {
    from_port       = 3000
    to_port         = 3000
    security_groups = [aws_security_group.alb.id]
  }
}

resource "aws_security_group" "alb" {
  name = "${var.project}-alb-sg"
  ingress {
    from_port   = 443
    to_port     = 443
    cidr_blocks = ["0.0.0.0/0"]
  }
  ingress {
    from_port   = 80
    to_port     = 80
    cidr_blocks = ["0.0.0.0/0"]
  }
}

variable "project" { type = string }
variable "aws_region" { type = string }
variable "vpc_id" { type = string }
variable "private_subnet_ids" { type = list(string) }
variable "public_subnet_ids" { type = list(string) }
variable "desired_count" { type = number, default = 2 }
```

## Common Scaffolding Commands

```bash
# Monorepo
pnpm create turbo@latest              # Initialize Turbo repo
pnpm exec turbo init                  # Add turbo to existing repo

# Next.js
pnpm create next-app@latest           # Create Next.js app
npx create-next-app --ts --tailwind   # With TypeScript + Tailwind

# Vite
pnpm create vite@latest              # Create Vite project

# Package scaffolding
pnpm add -w <package>                 # Add to workspace root
pnpm add -F <package>                 # Add to specific package (turbo)
```

## Best Practices

1. **Use pnpm workspaces** for efficient disk usage and faster installs
2. **Enable strict TypeScript** in shared configs — avoids runtime errors
3. **Use turbo.json cache** for CI speed — only rebuild changed packages
4. **Never commit .env files** — use .env.example as documentation
5. **Docker multi-stage builds** — minimize production image size
6. **Use Vercel/Fly/Railway** for quick deploys — Terraform for full control
7. **Test before merge** — require passing CI before merging PRs
8. **Atomic deploys** — use blue-green or canary for zero-downtime releases

## Guardrails

- Always run `pnpm lint` and `pnpm typecheck` before committing
- Ensure all secrets are injected at runtime, never hardcoded
- Use `--frozen-lockfile` in CI to prevent lockfile drift
- Pin Docker images to specific versions (not `latest`)
- Set up Dependabot or Renovate for automated dependency updates
