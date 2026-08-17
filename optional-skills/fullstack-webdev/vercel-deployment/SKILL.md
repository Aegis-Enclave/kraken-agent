---
name: vercel-deployment
description: "Vercel deployment for Next.js and full-stack apps — environment variables, preview deployments, edge functions, serverless."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [vercel, deployment, nextjs, serverless, edge-functions, preview-deployments]
    related_skills: [react-patterns, api-design, github-actions-templates]
---

# Vercel Deployment

## Project Setup

```bash
npm i -g vercel
vercel login
vercel init
```

```json
// vercel.json
{
  "framework": "nextjs",
  "buildCommand": "npm run build",
  "outputDirectory": ".next",
  "installCommand": "npm ci",
  "regions": ["iad1", "sfo1"],
  "headers": [
    {
      "source": "/api/(.*)",
      "headers": [
        { "key": "Cache-Control", "value": "no-cache, no-store, must-revalidate" }
      ]
    },
    {
      "source": "/static/(.*)",
      "headers": [
        { "key": "Cache-Control", "value": "public, max-age=31536000, immutable" }
      ]
    }
  ],
  "rewrites": [
    { "source": "/api/:path*", "destination": "/api/:path*" }
  ]
}
```

---

## Environment Variables

```bash
# CLI
vercel env add DATABASE_URL
vercel env add NEXT_PUBLIC_SENTRY_DSN
vercel env pull .env.local

# Protect sensitive vars in production
vercel env add JWT_SECRET production

# .env.local (gitignored)
DATABASE_URL=postgres://...
JWT_SECRET=...
```

Vercel dashboard: Project > Settings > Environment Variables

---

## Next.js App Router Deployment

```typescript
// app/api/posts/route.ts (Server Components + Route Handlers)
import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';

export async function GET(req: NextRequest) {
  const { searchParams } = new URL(req.url);
  const limit = Number(searchParams.get('limit') ?? '20');
  const cursor = searchParams.get('cursor');

  const posts = await prisma.post.findMany({
    take: limit + 1,
    where: { published: true },
    cursor: cursor ? { id: cursor } : undefined,
    orderBy: { createdAt: 'desc' },
    include: { author: { select: { name: true } } },
  });

  const hasNext = posts.length > limit;
  const data = hasNext ? posts.slice(0, -1) : posts;
  const nextCursor = hasNext ? data[data.length - 1].id : null;

  return NextResponse.json({ data, nextCursor });
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  // ... validation and creation
}
```

---

## Edge Functions

```typescript
// app/api/search/route.ts (Edge runtime)
import { NextRequest } from 'next/server';

export const runtime = 'edge';

export async function GET(req: NextRequest) {
  const { searchParams } = new URL(req.url);
  const query = searchParams.get('q');

  // Use edge-compatible fetch
  const results = await fetch(`https://api.search.com?q=${query}`, {
    cf: { cacheEverything: true, cacheTtl: 300 },
  }).then(r => r.json());

  return Response.json(results);
}
```

---

## Serverless Functions (Node.js Runtime)

```typescript
// For CPU-intensive or long-running tasks, use Node.js runtime
// app/api/process/route.ts

export const runtime = 'nodejs';

export async function POST(req: Request) {
  // Can run for up to 10s (API) or 60s (Background)
  const data = await req.json();
  const result = await processLargeDataset(data);
  return Response.json(result);
}
```

---

## Preview Deployments

Every PR gets an automatic preview URL. Custom domains supported:

```typescript
// vercel.json — set up preview branches
{
  "git": {
    "deploymentEnabled": {
      "main": true,
      "staging": true,
      "develop": true,
      "feature/*": false  // Disable auto-deploy for feature branches
    }
  }
}
```

---

## Monorepo Setup

```json
// vercel.json in apps/web
{
  "rootDirectory": "apps/web",
  "buildCommand": "npm run build --workspace=apps/web",
  "installCommand": "npm install"
}
```

Or use workspace-aware builds:
```bash
# Deploy specific workspace
vercel --cwd apps/web
```

---

## Cron Jobs

```json
// vercel.json
{
  "crons": [
    {
      "path": "/api/cron/daily-digest",
      "schedule": "0 8 * * *"
    }
  ]
}
```

```typescript
// app/api/cron/daily-digest/route.ts
export async function GET(req: NextRequest) {
  // Verify it's from Vercel Cron
  if (req.headers.get('x-vercel-cron') !== '1') {
    return new Response('Unauthorized', { status: 401 });
  }

  await sendDailyDigest();
  return Response.json({ ok: true });
}
```

---

## Monitoring

```bash
# CLI
vercel logs my-app
vercel domains
vercel ssl
vercel secrets list

# Dashboard
# vercel.com/dashboard > Project > Analytics / Logs / Metrics
```
