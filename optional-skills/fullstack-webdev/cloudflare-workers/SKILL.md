---
name: cloudflare-workers
description: "Cloudflare Workers deployment — D1, KV, R2, Pages, Durable Objects, Wrangler, TypeScript, edge computing patterns."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [cloudflare, workers, edge, serverless, d1, kv, r2, wrangler]
    related_skills: [docker-expert, api-design]
---

# Cloudflare Workers

## Project Setup

```bash
npm create cloudflare@latest my-worker
cd my-worker
npm run dev     # Local dev with Wrangler
npm run deploy # Deploy to edge
```

```typescript
// wrangler.toml
name = "my-worker"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[env.production]
name = "my-worker"

[[d1_databases]]
binding = "DB"
database_name = "my-db"
database_id = "xxx"

[[kv_namespaces]]
binding = "CACHE"
id = "xxx"
```

---

## Basic Handler

```typescript
// src/index.ts
interface Env {
  DB: D1Database;
  CACHE: KVNamespace;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === "/api/posts" && request.method === "GET") {
      return handleGetPosts(request, env);
    }

    if (url.pathname === "/api/posts" && request.method === "POST") {
      return handleCreatePost(request, env);
    }

    return new Response("Not Found", { status: 404 });
  },
};

async function handleGetPosts(request: Request, env: Env): Promise<Response> {
  const cached = await env.CACHE.get("posts:all");
  if (cached) {
    return new Response(cached, {
      headers: { "Content-Type": "application/json", "X-Cache": "HIT" },
    });
  }

  const { results } = await env.DB
    .prepare("SELECT * FROM posts ORDER BY created_at DESC LIMIT 20")
    .all();

  const data = JSON.stringify(results);
  await env.CACHE.put("posts:all", data, { expirationTtl: 300 });

  return new Response(data, {
    headers: { "Content-Type": "application/json", "X-Cache": "MISS" },
  });
}
```

---

## D1 Database

```sql
-- schema.sql
CREATE TABLE IF NOT EXISTS posts (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  author_id TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX idx_posts_author ON posts(author_id);
CREATE INDEX idx_posts_created ON posts(created_at);
```

```typescript
// Migrations
// npx wrangler d1 migrations apply my-db --local
// npx wrangler d1 migrations apply my-db --remote

// Queries
const stmt = env.DB
  .prepare("SELECT * FROM posts WHERE id = ?")
  .bind(postId);
const result = await stmt.first();

const { results } = await env.DB
  .prepare("SELECT * FROM posts WHERE author_id = ?")
  .bind(authorId)
  .all();
```

---

## KV Storage

```typescript
// Cache with TTL
await env.CACHE.put("user:session:" + sessionId, JSON.stringify(user), {
  expirationTtl: 86400, // 24 hours
});

// Get with default
const cached = await env.CACHE.get("post:" + postId, "json") as User | null;

// List keys
const list = await env.CACHE.list({ prefix: "posts:", limit: 100 });

// Delete
await env.CACHE.delete("post:" + postId);
```

---

## R2 Object Storage

```toml
# wrangler.toml
[[r2_buckets]]
binding = "ASSETS"
bucket_name = "my-assets"
```

```typescript
// Upload
const body = await request.arrayBuffer();
const key = `uploads/${crypto.randomUUID()}.png`;

await env.ASSETS.put(key, body, {
  httpMetadata: {
    contentType: request.headers.get("content-type") || "application/octet-stream",
  },
  customMetadata: { uploadedBy: userId },
});

return new Response(JSON.stringify({ key }), {
  headers: { "Content-Type": "application/json" },
});

// Serve
const object = await env.ASSETS.get(key);
if (!object) return new Response("Not found", { status: 404 });

return new Response(object.body, {
  headers: { "Content-Type": object.httpMetadata.contentType || "application/octet-stream" },
});
```

---

## Durable Objects

```typescript
// src/durable.ts
export class Counter implements DurableObject {
  private state: DurableObjectState;
  private count = 0;

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === "/increment") {
      this.count++;
      await this.state.storage.put("count", this.count);
      return new Response(String(this.count));
    }

    if (url.pathname === "/count") {
      const stored = await this.state.storage.get<number>("count");
      return new Response(String(stored ?? 0));
    }

    return new Response("Not found", { status: 404 });
  }
}
```

```toml
# wrangler.toml
[[durable_objects.bindings]]
name = "COUNTER"
class_name = "Counter"
```

```typescript
// From worker
const id = env.COUNTER.idFromName("global-counter");
const stub = env.COUNTER.get(id);
const count = await stub.fetch("http://localhost/count");
```

---

## Middleware Pattern

```typescript
type Middleware = (
  request: Request,
  env: Env,
  ctx: ExecutionContext
) => Promise<Response | null>;

async function withAuth(request: Request, env: Env): Promise<Response | null> {
  const token = request.headers.get("Authorization")?.replace("Bearer ", "");
  if (!token) {
    return new Response(JSON.stringify({ error: "Unauthorized" }), {
      status: 401,
      headers: { "Content-Type": "application/json" },
    });
  }
  const user = await verifyToken(token, env.JWT_SECRET);
  if (!user) {
    return new Response(JSON.stringify({ error: "Invalid token" }), {
      status: 401,
      headers: { "Content-Type": "application/json" },
    });
  }
  return null; // Continue
}

async function handle(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
  // Your handler
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const authError = await withAuth(request, env);
    if (authError) return authError;

    return handle(request, env, ctx);
  },
};
```

---

## Cron Triggers

```toml
# wrangler.toml
[triggers]
crons = ["*/15 * * * *", "0 */6 * * *"]  # Every 15 min, every 6 hours
```

```typescript
export default {
  async scheduled(controller: ScheduledController, env: Env, ctx: ExecutionContext): Promise<void> {
    switch (controller.cron) {
      case "*/15 * * * *":
        await cleanupExpiredSessions(env);
        break;
      case "0 */6 * * *":
        await generateReports(env);
        break;
    }
  },
};
```

---

## Environment Variables & Secrets

```bash
# Set secrets (non-sensitive config)
wrangler secret put JWT_SECRET
# Paste your secret value

# Environment-specific vars
wrangler dev --var SOME_VAR:value
# Or in wrangler.toml [vars]
[vars]
ENVIRONMENT = "production"
MAX_ITEMS = "100"
```
