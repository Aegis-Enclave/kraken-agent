---
name: redis-cache
description: "Redis caching patterns for web apps — cache-aside, sessions, rate limiting, pub/sub, distributed locks, BullMQ integration."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [redis, caching, sessions, rate-limiting, pubsub, distributed-lock, ioredis]
    related_skills: [nodejs-backend-patterns, server-management]
---

# Redis Cache Patterns

## When to Use Redis

- Session storage
- API response caching
- Rate limiting
- Job queues (BullMQ)
- Real-time features (pub/sub)
- Leaderboards / sorted sets
- Distributed locks

**Don't** use Redis as your primary database. It's fast but volatile.

---

## Client Setup

```javascript
// ioredis (preferred — better API, cluster support)
import Redis from 'ioredis';

const redis = new Redis(process.env.REDIS_URL, {
  maxRetriesPerRequest: 3,
  retryDelayOnFailover: 100,
  enableReadyCheck: true,
  lazyConnect: true,
});

redis.on('error', (err) => logger.error({ err }, 'Redis connection error'));
redis.on('connect', () => logger.info('Redis connected'));

// For clusters
import RedisCluster from 'ioredis/lib/cluster';

// Connection string: redis://user:pass@host:port/db
```

---

## Key Naming Conventions

```
# Pattern: {scope}:{entity}:{id}:{field}
user:session:{sessionId}
user:profile:{userId}
post:cache:{postId}
rate:api:{userId}
lock:payment:{orderId}
job:email:{jobId}

# Sets
user:{userId}:followers
tag:popular:{tagId}:posts

# Sorted sets (with scores)
leaderboard:posts:2026
rate:sliding:{userId}
```

Always use colons as delimiters. Keep names readable but short.

---

## Cache-Aside Pattern

```javascript
// Read-through cache
async function getPost(postId) {
  const cacheKey = `post:cache:${postId}`;

  // 1. Check cache
  const cached = await redis.get(cacheKey);
  if (cached) {
    return JSON.parse(cached);
  }

  // 2. Fetch from DB
  const post = await db.posts.findUnique({ where: { id: postId } });
  if (!post) return null;

  // 3. Store in cache
  await redis.setex(cacheKey, 300, JSON.stringify(post)); // 5 min TTL

  return post;
}

// Write-through (update cache on write)
async function createPost(data) {
  const post = await db.posts.create({ data });

  await redis.setex(`post:cache:${post.id}`, 300, JSON.stringify(post));

  // Invalidate list caches
  await redis.del('posts:recent:list');

  return post;
}

// Cache invalidation
async function deletePost(postId) {
  await db.posts.delete({ where: { id: postId } });
  await redis.del(`post:cache:${postId}`);
  await redis.del('posts:recent:list');
}
```

---

## Sliding Window Rate Limiting

```javascript
// src/middleware/slidingWindowRateLimit.js
import Redis from 'ioredis';

export function slidingWindowRateLimit(redis, options) {
  const { windowMs, maxRequests } = options;

  return async (req, res, next) => {
    const key = `rate:sliding:${req.ip}:${req.path}`;
    const now = Date.now();
    const windowStart = now - windowMs;

    const pipeline = redis.pipeline();
    pipeline.zremrangebyscore(key, 0, windowStart); // Remove old entries
    pipeline.zadd(key, now, `${now}:${Math.random()}`); // Add current request
    pipeline.zcard(key); // Count requests
    pipeline.pexpire(key, windowMs); // Auto-cleanup

    const results = await pipeline.exec();
    const requestCount = results[2][1];

    res.set({
      'X-RateLimit-Limit': maxRequests,
      'X-RateLimit-Remaining': Math.max(0, maxRequests - requestCount),
      'X-RateLimit-Reset': Math.ceil((now + windowMs) / 1000),
    });

    if (requestCount > maxRequests) {
      return res.status(429).json({
        error: { code: 'RATE_LIMITED', message: 'Too many requests' }
      });
    }

    next();
  };
}
```

---

## Session Storage

```javascript
import session from 'express-session';
import RedisStore from 'connect-redis';
import { redis } from './db.js';

app.use(session({
  store: new RedisStore({ client: redis, prefix: 'sess:' }),
  secret: process.env.SESSION_SECRET,
  resave: false,
  saveUninitialized: false,
  cookie: {
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    maxAge: 7 * 24 * 60 * 60 * 1000, // 7 days
    sameSite: 'lax',
  },
}));
```

---

## Distributed Locks (Redlock)

```javascript
// Simple lock
async function acquireLock(key, ttlMs = 30000) {
  const lockKey = `lock:${key}`;
  const lockValue = crypto.randomUUID();
  const acquired = await redis.set(lockKey, lockValue, 'PX', ttlMs, 'NX');
  if (acquired === 'OK') {
    return { release: () => redis.del(lockKey), value: lockValue };
  }
  return null;
}

async function processWithLock(orderId, fn) {
  const lock = await acquireLock(`payment:${orderId}`, 60000);
  if (!lock) throw new Error('Could not acquire lock');

  try {
    return await fn();
  } finally {
    await lock.release();
  }
}
```

---

## Pub/Sub

```javascript
// Publisher
redis.publish('user:events', JSON.stringify({
  type: 'USER_UPDATED',
  userId: user.id,
  timestamp: Date.now(),
}));

// Subscriber
const subscriber = new Redis();
subscriber.subscribe('user:events');
subscriber.on('message', (channel, message) => {
  const event = JSON.parse(message);
  handleUserEvent(event);
});
```

---

## TTL Strategy

| Data Type | TTL | Reason |
|-----------|-----|--------|
| API response cache | 5-15 min | Balance freshness vs speed |
| User session | 7-30 days | Session expiry |
| Rate limit window | window duration | Auto-cleanup |
| Job lock | 30-60s | Prevent deadlocks |
| Temporary token | token lifetime | Match token expiry |
| Feature flag | 5-60 min | Allow quick toggles |

---

## Memory Management

```bash
# redis.conf
maxmemory 256mb
maxmemory-policy allkeys-lru  # Evict least recently used when full
maxmemory-samples 5            # Sampling for LRU
```

Monitor with `redis-cli INFO memory` — watch `used_memory_human` and `maxmemory_human`.

---

## Common Pitfalls

1. **Don't store huge objects** — Redis is fast but has a 512MB value limit
2. **Always set TTL** — keys without TTL are memory leaks
3. **Pipeline commands** — batch multiple commands to reduce round trips
4. **Use SCAN not KEYS** — `KEYS *` blocks the server
5. **Connection pooling** — one client per app, share the connection
6. **Handle reconnection** — ioredis auto-reconnects, but check errors
