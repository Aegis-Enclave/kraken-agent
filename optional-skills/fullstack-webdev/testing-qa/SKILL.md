---
name: testing-qa
description: "Comprehensive testing strategy for web apps — unit, integration, E2E (Playwright), Lighthouse CI, test coverage, CI integration."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [testing, playwright, vitest, jest, e2e, unit-test, integration-test, lighthouse, coverage]
    related_skills: [react-patterns, nodejs-backend-patterns, github-actions-templates]
---

# Testing QA

## Testing Pyramid

```
       /\
      /E2E\        ← Few, slow, expensive, high confidence
     /------\
    /Integr. \      ← Medium count, medium speed
   /----------\
  /  Unit Tests \  ← Many, fast, cheap, isolated
 /--------------\
```

**Unit**: pure functions, utils, components in isolation
**Integration**: API routes, DB interactions, service layers
**E2E**: critical user flows (login, checkout, forms)

---

## Vitest (Node.js + Vite)

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      thresholds: { lines: 80, functions: 80, branches: 70 },
    },
  },
});
```

```typescript
// tests/setup.ts
import '@testing-library/jest-dom';
import { afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { server } from './mocks/server';

afterEach(() => { cleanup(); });

// Mock browser APIs
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
```

```typescript
// tests/mocks/server.ts
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

export const server = setupServer(
  http.get('/api/users', () => HttpResponse.json([{ id: 1, name: 'Alice' }])),
  http.post('/api/users', () => HttpResponse.json({ id: 2, name: 'Bob' }, { status: 201 })),
);

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

```typescript
// Unit test example
import { describe, it, expect, vi } from 'vitest';
import { formatCurrency, calculateTax } from '../utils/money';

describe('money utils', () => {
  describe('formatCurrency', () => {
    it('formats USD correctly', () => {
      expect(formatCurrency(1234.56, 'USD')).toBe('$1,234.56');
    });

    it('handles zero', () => {
      expect(formatCurrency(0, 'USD')).toBe('$0.00');
    });

    it('handles negative amounts', () => {
      expect(formatCurrency(-50, 'USD')).toBe('-$50.00');
    });
  });
});

// Component test example
import { render, screen, fireEvent } from '@testing-library/react';
import { LoginForm } from './LoginForm';

describe('LoginForm', () => {
  it('shows validation errors on empty submit', async () => {
    render(<LoginForm onSubmit={vi.fn()} />);

    fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

    expect(await screen.findByText(/email is required/i)).toBeInTheDocument();
  });

  it('calls onSubmit with form data', async () => {
    const onSubmit = vi.fn();
    render(<LoginForm onSubmit={onSubmit} />);

    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: 'alice@example.com' },
    });
    fireEvent.change(screen.getByLabelText(/password/i), {
      target: { value: 'secret123' },
    });
    fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

    expect(onSubmit).toHaveBeenCalledWith({
      email: 'alice@example.com',
      password: 'secret123',
    });
  });
});
```

---

## Playwright E2E

```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [['html', { open: 'never' }], ['list']],
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 13'] },
    },
  ],
});
```

```typescript
// tests/e2e/auth.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('user can sign up with email and password', async ({ page }) => {
    await page.goto('/signup');

    await page.getByLabel(/email/i).fill('newuser@example.com');
    await page.getByLabel(/^password$/i).fill('SecurePass123!');
    await page.getByLabel(/confirm password/i).fill('SecurePass123!');
    await page.getByRole('button', { name: /create account/i }).click();

    await expect(page).toHaveURL('/dashboard');
    await expect(page.getByText(/welcome/i)).toBeVisible();
  });

  test('shows validation errors for weak passwords', async ({ page }) => {
    await page.goto('/signup');

    await page.getByLabel(/email/i).fill('user@example.com');
    await page.getByLabel(/^password$/i).fill('123');
    await page.getByRole('button', { name: /create account/i }).click();

    await expect(page.getByText(/at least 8 characters/i)).toBeVisible();
  });

  test('user can log in and log out', async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('alice@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in/i }).click();

    await expect(page).toHaveURL('/dashboard');

    await page.getByRole('button', { name: /avatar/i }).click();
    await page.getByRole('menuitem', { name: /log out/i }).click();

    await expect(page).toHaveURL('/login');
  });
});

test.describe('Dashboard', () => {
  test.use({ storageState: 'tests/e2e/auth.json' }); // Pre-authenticated

  test('displays user data and allows navigation', async ({ page }) => {
    await page.goto('/dashboard');
    await expect(page.getByRole('heading', { name: /dashboard/i })).toBeVisible();

    await page.getByRole('link', { name: /settings/i }).click();
    await expect(page).toHaveURL('/settings');
  });
});
```

---

## Lighthouse CI

```yaml
# lighthouse.config.js
module.exports = {
  ci: {
    collect: {
      url: [
        'http://localhost:3000/',
        'http://localhost:3000/dashboard',
        'http://localhost:3000/settings',
      ],
      startServerCommand: 'npm run start',
      startServerReadyPattern: 'Server running',
      startServerReadyTimeout: 30000,
    },
    assert: {
      assertions: {
        'categories:performance': ['error', { minScore: 0.8 }],
        'categories:accessibility': ['error', { minScore: 0.9 }],
        'categories:best-practices': ['error', { minScore: 0.85 }],
        'categories:seo': ['error', { minScore: 0.9 }],
        'first-contentful-paint': ['warn', { maxNumericValue: 2000 }],
        'largest-contentful-paint': ['error', { maxNumericValue: 4000 }],
        'cumulative-layout-shift': ['error', { maxNumericValue: 0.1 }],
        'total-blocking-time': ['error', { maxNumericValue: 500 }],
      },
    },
    upload: {
      target: 'lhci',
    },
  },
};
```

---

## Coverage Reporting

```bash
# Coverage gate in CI
npx vitest run --coverage
npx vitest coverage --coverage.reporter=text-summary
```

GitHub Actions integration:
```yaml
- name: Check coverage
  run: |
    npx vitest run --coverage
    npx jest-coverage-badges output=coverage/badges
  if: matrix.node == '20'
```

---

## Test Naming Convention

```
describe(UnitOfWork, () => {
  describe(methodOrScenario, () => {
    it('should [expected behavior] when [condition]', () => { ... });
    it('should return null when user does not exist', () => { ... });
  });
});
```

Be descriptive. The test name is documentation.
