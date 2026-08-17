---
name: nodejs-backend-patterns
description: "Production Node.js backend patterns — Express structure, async errors, middleware, security, logging, PM2, BullMQ, testing."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [nodejs, express, backend, production, middleware, security, pm2, bullmq]
    related_skills: [api-design, docker-expert, github-actions-templates, redis-cache]
---

# Node.js Backend Patterns

## Project Structure

```
src/
  app.js              # Express app setup (no listen() here)
  server.js           # Entry point — creates app, listens
  config/
    index.js          # env validation with zod
    database.js       # Prisma/DB client
  routes/
    users.js
    posts.js
  controllers/
    users.js
    posts.js
  services/
    users.js
    posts.js
  middleware/
    auth.js
    errorHandler.js
    rateLimiter.js
    validator.js
  utils/
    errors.js         # Custom error classes
    logger.js
  tests/
    users.test.js
```

Never put route logic in controllers — put it in services. Controllers should only call services.

---

## App Setup

```javascript
// src/app.js
import 'express-async-errors';
import express from 'express';
import helmet from 'helmet';
import cors from 'cors';
import { errorHandler } from './middleware/errorHandler.js';
import { rateLimiter } from './middleware/rateLimiter.js';
import usersRouter from './routes/users.js';
import postsRouter from './routes/posts.js';

const app = express();

app.use(helmet());
app.use(cors({ origin: process.env.ALLOWED_ORIGINS?.split(',') }));
app.use(express.json({ limit: '10mb' }));
app.use(rateLimiter);

// Routes
app.use('/users', usersRouter);
app.use('/posts', postsRouter);
app.get('/health', (req, res) => res.json({ status: 'ok' }));

app.use(errorHandler);

export default app;
```

```javascript
// src/server.js
import app from './app.js';
import { config } from './config/index.js';
import { logger } from './utils/logger.js';

const port = config.PORT;

const server = app.listen(port, () => {
  logger.info(`Server running on port ${port}`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  logger.info('SIGTERM received, shutting down gracefully');
  server.close(() => {
    logger.info('Server closed');
    process.exit(0);
  });
});
```

---

## Async Error Handling

Use `express-async-errors` to avoid try/catch in every route:

```javascript
// Top of app.js (before routes)
import 'express-async-errors';

app.post('/users', async (req, res) => {
  // No try/catch needed — express-async-errors catches and passes to error handler
  const user = await userService.create(req.body);
  res.status(201).json(user);
});
```

---

## Custom Error Classes

```javascript
// src/utils/errors.js
export class AppError extends Error {
  constructor(message, statusCode, code) {
    super(message);
    this.statusCode = statusCode;
    this.code = code;
    this.isOperational = true;
    Error.captureStackTrace(this, this.constructor);
  }
}

export class NotFoundError extends AppError {
  constructor(resource = 'Resource') {
    super(`${resource} not found`, 404, 'NOT_FOUND');
  }
}

export class ValidationError extends AppError {
  constructor(details) {
    super('Validation failed', 422, 'VALIDATION_ERROR');
    this.details = details;
  }
}

export class UnauthorizedError extends AppError {
  constructor() {
    super('Authentication required', 401, 'UNAUTHORIZED');
  }
}

export class ForbiddenError extends AppError {
  constructor() {
    super('Insufficient permissions', 403, 'FORBIDDEN');
  }
}
```

---

## Error Handler Middleware

```javascript
// src/middleware/errorHandler.js
import { AppError } from '../utils/errors.js';
import { logger } from '../utils/logger.js';
import { config } from '../config/index.js';

export const errorHandler = (err, req, res, next) => {
  const requestId = req.headers['x-request-id'] || crypto.randomUUID();

  if (err.isOperational) {
    return res.status(err.statusCode).json({
      error: {
        code: err.code,
        message: err.message,
        details: err.details,
        request_id: requestId,
      },
    });
  }

  // Unknown error — log and hide details in production
  logger.error({ err, requestId }, 'Unhandled error');
  res.status(500).json({
    error: {
      code: 'INTERNAL_ERROR',
      message: config.NODE_ENV === 'production'
        ? 'An unexpected error occurred'
        : err.message,
      request_id: requestId,
    },
  });
};
```

---

## Authentication Middleware

```javascript
// src/middleware/auth.js
import { UnauthorizedError, ForbiddenError } from '../utils/errors.js';
import { verifyToken } from '../services/auth.js';

export const authenticate = (req, res, next) => {
  const authHeader = req.headers.authorization;
  if (!authHeader?.startsWith('Bearer ')) {
    throw new UnauthorizedError();
  }
  const token = authHeader.slice(7);
  req.user = verifyToken(token);
  next();
};

export const authorize = (...roles) => (req, res, next) => {
  if (!roles.includes(req.user.role)) {
    throw new ForbiddenError();
  }
  next();
};
```

---

## Environment Config

```javascript
// src/config/index.js
import 'dotenv/config';
import { z } from 'zod';

const envSchema = z.object({
  NODE_ENV: z.enum(['development', 'test', 'production']).default('development'),
  PORT: z.coerce.number().min(1).max(65535).default(3000),
  DATABASE_URL: z.string().url(),
  JWT_SECRET: z.string().min(32),
  JWT_EXPIRES_IN: z.string().default('7d'),
  REDIS_URL: z.string().url().optional(),
  ALLOWED_ORIGINS: z.string().optional(),
  RATE_LIMIT_MAX: z.coerce.number().default(100),
  RATE_LIMIT_WINDOW_MS: z.coerce.number().default(60000),
});

export const config = envSchema.parse(process.env);
```

---

## Logging

Use Pino for structured JSON logs:

```javascript
// src/utils/logger.js
import pino from 'pino';

export const logger = pino({
  level: process.env.LOG_LEVEL || 'info',
  formatters: {
    level: (label) => ({ level: label }),
  },
  timestamp: pino.stdTimeFunctions.isoTime,
  ...(process.env.NODE_ENV === 'production' ? {} : {
    transport: { target: 'pino-pretty', options: { colorize: true } },
  }),
});

// Usage
logger.info({ userId: req.user.id, path: req.path }, 'Request received');
logger.error({ err }, 'Database connection failed');
```

---

## Rate Limiting

```javascript
// src/middleware/rateLimiter.js
import rateLimit from 'express-rate-limit';
import { config } from '../config/index.js';

export const rateLimiter = rateLimit({
  windowMs: config.RATE_LIMIT_WINDOW_MS,
  max: config.RATE_LIMIT_MAX,
  standardHeaders: true,
  legacyHeaders: false,
  handler: (req, res) => {
    res.status(429).json({
      error: {
        code: 'RATE_LIMITED',
        message: 'Too many requests, please try again later',
      },
    });
  },
});

// Stricter limit for auth endpoints
export const authRateLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 5,
  skipSuccessfulRequests: true,
});
```

---

## BullMQ Job Queues

```javascript
// src/queues/email.js
import { Queue, Worker } from 'bullmq';
import { redisConnection } from '../config/database.js';

export const emailQueue = new Queue('email', { connection: redisConnection });

export const emailWorker = new Worker('email', async (job) => {
  const { to, subject, template } = job.data;
  await sendEmail({ to, subject, template });
  job.updateProgress(100);
}, {
  connection: redisConnection,
  concurrency: 5,
});

emailWorker.on('failed', (job, err) => {
  logger.error({ jobId: job.id, err }, 'Email job failed');
});

// Enqueue
await emailQueue.add('welcome', {
  to: user.email,
  subject: 'Welcome!',
  template: 'welcome',
}, {
  attempts: 3,
  backoff: { type: 'exponential', delay: 1000 },
});
```

---

## Health Check

```javascript
// src/routes/health.js
import { Router } from 'express';
import { prisma } from '../config/database.js';

const router = Router();

router.get('/health', async (req, res) => {
  try {
    await prisma.$queryRaw`SELECT 1`;
    res.json({
      status: 'ok',
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
    });
  } catch (err) {
    res.status(503).json({ status: 'unhealthy', error: err.message });
  }
});

router.get('/ready', async (req, res) => {
  // Readiness probe — check all dependencies
  const checks = await Promise.allSettled([
    prisma.$queryRaw`SELECT 1`,
    redis.ping(),
  ]);
  const allReady = checks.every(c => c.status === 'fulfilled');
  res.status(allReady ? 200 : 503).json({ ready: allReady });
});

export default router;
```

---

## Testing with Supertest

```javascript
// tests/users.test.js
import request from 'supertest';
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import app from '../src/app.js';
import { prisma } from '../src/config/database.js';

describe('Users API', () => {
  beforeAll(async () => { /* seed test data */ });
  afterAll(async () => { await prisma.$disconnect(); });

  describe('GET /users', () => {
    it('returns paginated users', async () => {
      const res = await request(app)
        .get('/users?limit=10')
        .set('Authorization', `Bearer ${testToken}`);

      expect(res.status).toBe(200);
      expect(res.body.data).toBeInstanceOf(Array);
      expect(res.body.pagination).toBeDefined();
    });
  });

  describe('POST /users', () => {
    it('creates a user with valid data', async () => {
      const res = await request(app)
        .post('/users')
        .send({ email: 'test@example.com', name: 'Test' });

      expect(res.status).toBe(201);
      expect(res.body.email).toBe('test@example.com');
    });

    it('returns 422 for invalid data', async () => {
      const res = await request(app)
        .post('/users')
        .send({ email: 'not-an-email' });

      expect(res.status).toBe(422);
      expect(res.body.error.code).toBe('VALIDATION_ERROR');
    });
  });
});
```

---

## Security Checklist

- `helmet()` — security headers
- `cors()` — configured origins only
- `express-rate-limit` — against brute force
- `hpp` — against parameter pollution
- Input validation with Zod at every endpoint
- Never expose stack traces in production
- Sanitize log inputs (no user-controlled content in logs without sanitization)
- Use `crypto.randomUUID()` for IDs (not sequential integers)
- Set `httpOnly`, `secure`, `sameSite` on cookies
- Validate file uploads (type, size, filename sanitization)
