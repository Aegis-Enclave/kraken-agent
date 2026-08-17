---
name: sentry-integration
description: "Sentry error tracking and performance monitoring for web apps — React, Node.js, error boundaries, tracing, source maps."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [sentry, error-tracking, monitoring, observability, performance, react, nodejs]
    related_skills: [nodejs-backend-patterns, react-patterns, docker-expert]
---

# Sentry Integration

## Node.js Backend

```bash
npm install @sentry/node
```

```typescript
// src/instrument.ts
import * as Sentry from '@sentry/node';

Sentry.init({
  dsn: process.env.SENTRY_DSN,
  environment: process.env.NODE_ENV,
  release: process.env.HEROKU_SLUG_COMMIT || 'unknown',
  sampleRate: 0.1, // 10% of transactions in production
  tracesSampleRate: 0.1,
  profilesSampleRate: 0.1,
  maxBreadcrumbs: 50,
  attachStacktrace: process.env.NODE_ENV !== 'production',
  beforeSend(event) {
    // Filter out noise
    if (event.exception?.values?.[0]?.type === 'ValidationError') {
      return null; // Skip validation errors
    }
    return event;
  },
  ignoreErrors: [
    /Network Error/i,
    /ECONNREFUSED/i,
    /Non-Error promise rejection/,
  ],
});
```

```typescript
// src/server.ts
import * as Sentry from '@sentry/node';
import { errorHandler } from './middleware/errorHandler.js';

app.use(Sentry.Handlers.requestHandler());

// Your routes...

app.use(Sentry.Handlers.errorHandler());

app.use(errorHandler);

// Flush events before exit
process.on('SIGTERM', () => {
  Sentry.close(2000).then(() => process.exit(0));
});
```

---

## Express Error Tracking

```typescript
// Automatic error capture (Sentry captures unhandled errors automatically)
// But for explicit capture:
import * as Sentry from '@sentry/node';

app.post('/api/upload', uploadMiddleware, async (req, res, next) => {
  try {
    const result = await processUpload(req.file);
    res.json(result);
  } catch (err) {
    Sentry.captureException(err, {
      extra: {
        fileName: req.file?.originalname,
        fileSize: req.file?.size,
      },
    });
    next(err);
  }
});
```

---

## React Frontend

```bash
npm install @sentry/react @sentry/tracing
```

```typescript
// src/instrument.tsx
import * as Sentry from '@sentry/react';
import { BrowserTracing } from '@sentry/tracing';

Sentry.init({
  dsn: process.env.VITE_SENTRY_DSN,
  environment: import.meta.env.MODE,
  integrations: [
    new BrowserTracing({
      tracingOrigins: ['localhost', /^https:\/\/api\.myapp\.com/],
    }),
    new Sentry.Replay({
      maskAllText: false,
      blockAllMedia: false,
    }),
  ],
  tracesSampleRate: 0.1,
  replaysSessionSampleRate: 0.05,
  replaysOnErrorSampleRate: 1.0, // Always capture replay on error
});
```

```typescript
// src/main.tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import * as Sentry from '@sentry/react';
import App from './App';

Sentry.init({ dsn: import.meta.env.VITE_SENTRY_DSN });

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

---

## React Error Boundary

```typescript
// src/components/ErrorBoundary.tsx
import { Component, type ReactNode, type ErrorInfo } from 'react';
import * as Sentry from '@sentry/react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    Sentry.captureException(error, { extra: { componentStack: errorInfo.componentStack } });
    this.props.onError?.(error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback ?? (
        <div className="error-fallback">
          <h1>Something went wrong</h1>
          <button onClick={() => this.setState({ hasError: false })}>
            Try again
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

// Usage
<ErrorBoundary fallback={<GlobalErrorPage />}>
  <Router />
</ErrorBoundary>
```

---

## Source Maps

```yaml
# GitHub Actions - Upload source maps
# .github/workflows/sentry.yml
- name: Upload source maps
  run: |
    npx sentry-cli releases files "${{ github.sha }}" upload-sourcemaps ./dist
    npx sentry-cli releases finalize "${{ github.sha }}"
  env:
    SENTRY_AUTH_TOKEN: ${{ secrets.SENTRY_AUTH_TOKEN }}
    SENTRY_ORG: myorg
    SENTRY_PROJECT: myapp
```

```json
// vite.config.ts (build step)
import { defineConfig } from 'vite';
import { sentryVitePlugin } from '@sentry/vite-plugin';

export default defineConfig({
  build: {
    sourcemap: true,
  },
  plugins: [
    sentryVitePlugin({
      org: 'myorg',
      project: 'myapp',
      authToken: process.env.SENTRY_AUTH_TOKEN,
    }),
  ],
});
```

---

## Performance Monitoring

```typescript
// Manual transaction for background tasks
import * as Sentry from '@sentry/node';

async function processQueue() {
  const transaction = Sentry.startTransaction({ name: 'processEmailQueue' });

  try {
    const jobs = await emailQueue.getJobs();
    for (const job of jobs) {
      const childSpan = transaction.startChild({ op: 'processJob', data: { jobId: job.id } });
      await processJob(job);
      childSpan.finish();
    }
    transaction.setStatus(SpanStatus.Ok);
  } catch (err) {
    transaction.setStatus(SpanStatus.InternalError);
    Sentry.captureException(err);
  } finally {
    transaction.finish();
  }
}
```

---

## User Context

```typescript
// Set user in frontend after login
import * as Sentry from '@sentry/react';

Sentry.setUser({
  id: user.id,
  email: user.email,
  username: user.name,
  ip_address: '{{auto}}', // Auto-capture IP
});

// Clear on logout
Sentry.setUser(null);
```

---

## Alerts & Issues

Create alerts in Sentry UI or via code:

```typescript
// Trigger a check-in (heartbeat)
import * as Sentry from '@sentry/node';

setInterval(() => {
  Sentry.captureCheckIn(
    { monitorSlug: 'daily-batch-job', status: 'in_progress' },
    { schedule: { type: 'interval', value: 1, unit: 'day' } }
  );
}, 24 * 60 * 60 * 1000);
```

---

## Docker Source Maps

```dockerfile
# In Dockerfile, before build
COPY . .
RUN SENTRY_ORG=myorg SENTRY_PROJECT=myapp SENTRY_AUTH_TOKEN=${SENTRY_AUTH_TOKEN} npm run build

# NOT the inverse — build must happen after COPY, before the multi-stage copy
```
