---
name: auth-implementation-patterns
description: "Authentication and authorization patterns — JWT, sessions, OAuth2, password hashing, RBAC, middleware, secure cookies, refresh tokens."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [auth, jwt, oauth2, session, bcrypt, argon2, rbac, password, security]
    related_skills: [nodejs-backend-patterns, api-design, web-security-testing]
---

# Auth Implementation Patterns

## Password Hashing

**Always** use a dedicated password hashing algorithm — never plain hashing.

```typescript
// Argon2id (recommended — use this for new projects)
import argon2 from 'argon2';

async function hashPassword(password: string): Promise<string> {
  return argon2.hash(password, {
    type: argon2.argon2id,
    memoryCost: 65536,    // 64MB
    timeCost: 3,          // 3 iterations
    parallelism: 4,
  });
}

async function verifyPassword(password: string, hash: string): Promise<boolean> {
  try {
    return await argon2.verify(hash, password);
  } catch {
    return false;
  }
}

// bcrypt (legacy but still acceptable)
import bcrypt from 'bcrypt';

const hash = await bcrypt.hash(password, 12); // Cost factor 12
const valid = await bcrypt.compare(password, hash);
```

---

## JWT Tokens

```typescript
// lib/auth.ts
import jwt from 'jsonwebtoken';
import { config } from '../config/index.js';

interface TokenPayload {
  userId: string;
  email: string;
  role: string;
}

export function signAccessToken(payload: TokenPayload): string {
  return jwt.sign(payload, config.JWT_SECRET, {
    expiresIn: '15m',
    issuer: 'myapp',
    audience: 'myapp-client',
  });
}

export function signRefreshToken(payload: { userId: string }): string {
  return jwt.sign(payload, config.JWT_REFRESH_SECRET, {
    expiresIn: '7d',
    iss: 'myapp',
  });
}

export function verifyAccessToken(token: string): TokenPayload {
  return jwt.verify(token, config.JWT_SECRET, {
    issuer: 'myapp',
    audience: 'myapp-client',
  }) as TokenPayload;
}

export function verifyRefreshToken(token: string): { userId: string } {
  return jwt.verify(token, config.JWT_REFRESH_SECRET, {
    issuer: 'myapp',
  }) as { userId: string };
}
```

---

## Cookie-Based Sessions

```typescript
// Secure cookie settings
const SESSION_COOKIE_OPTIONS = {
  httpOnly: true,       // JavaScript can't read it (XSS protection)
  secure: process.env.NODE_ENV === 'production', // HTTPS only in prod
  sameSite: 'strict',   // CSRF protection (use 'lax' if doing cross-origin)
  maxAge: 7 * 24 * 60 * 60 * 1000, // 7 days
  path: '/',
  signed: true,
};

// Signing cookies
import cookie from 'cookie';

res.setHeader('Set-Cookie', cookie.serialize('session_id', sessionId, {
  ...SESSION_COOKIE_OPTIONS,
  secrets: [config.COOKIE_SECRET],
}));

// Parsing cookies
import cookie from 'cookie';
const cookies = cookie.parse(req.headers.cookie || '');
const sessionId = cookies.session_id;
```

---

## Refresh Token Rotation

```typescript
// POST /auth/refresh
async function refreshTokens(req: Request, res: Response) {
  const refreshToken = req.cookies.refresh_token;
  if (!refreshToken) {
    throw new UnauthorizedError();
  }

  // Verify refresh token
  const payload = verifyRefreshToken(refreshToken);
  const user = await userService.findById(payload.userId);
  if (!user || user.tokenVersion !== payload.tokenVersion) {
    throw new UnauthorizedError(); // Token reuse detected
  }

  // Rotate: issue new refresh token (invalidate old one)
  await userService.incrementTokenVersion(user.id);

  const newRefreshToken = signRefreshToken({
    userId: user.id,
    tokenVersion: user.tokenVersion + 1,
  });

  const accessToken = signAccessToken({
    userId: user.id,
    email: user.email,
    role: user.role,
  });

  res.cookie('refresh_token', newRefreshToken, { ...SESSION_COOKIE_OPTIONS, httpOnly: true });
  res.json({ accessToken });
}
```

---

## OAuth2 (Google Example)

```typescript
// 1. Redirect to Google
app.get('/auth/google', (req, res) => {
  const params = new URLSearchParams({
    client_id: config.GOOGLE_CLIENT_ID,
    redirect_uri: `${config.BASE_URL}/auth/google/callback`,
    response_type: 'code',
    scope: 'openid email profile',
    access_type: 'offline',
    prompt: 'consent',
  });
  res.redirect(`https://accounts.google.com/o/oauth2/v2/auth?${params}`);
});

// 2. Handle callback
app.get('/auth/google/callback', async (req, res) => {
  const { code } = req.query;

  // Exchange code for tokens
  const tokens = await exchangeCode(code, config.GOOGLE_CLIENT_ID, config.GOOGLE_CLIENT_SECRET);

  // Get user info
  const userInfo = await fetch('https://www.googleapis.com/oauth2/v2/userinfo', {
    headers: { Authorization: `Bearer ${tokens.access_token}` },
  }).then(r => r.json());

  // Find or create user
  let user = await userService.findByEmail(userInfo.email);
  if (!user) {
    user = await userService.create({
      email: userInfo.email,
      name: userInfo.name,
      avatar: userInfo.picture,
      provider: 'google',
      providerId: userInfo.id,
    });
  }

  // Issue session
  const accessToken = signAccessToken({ userId: user.id, email: user.email, role: user.role });
  res.cookie('access_token', accessToken, SESSION_COOKIE_OPTIONS);
  res.redirect('/dashboard');
});
```

---

## RBAC (Role-Based Access Control)

```typescript
// src/types/auth.ts
export enum Role {
  ADMIN = 'admin',
  EDITOR = 'editor',
  USER = 'user',
  GUEST = 'guest',
}

export enum Permission {
  POST_CREATE = 'post:create',
  POST_EDIT = 'post:edit',
  POST_DELETE = 'post:delete',
  USER_MANAGE = 'user:manage',
  SETTINGS_VIEW = 'settings:view',
  SETTINGS_EDIT = 'settings:edit',
}

// Role → Permissions mapping
export const ROLE_PERMISSIONS: Record<Role, Permission[]> = {
  [Role.ADMIN]: Object.values(Permission),
  [Role.EDITOR]: [Permission.POST_CREATE, Permission.POST_EDIT, Permission.SETTINGS_VIEW],
  [Role.USER]: [Permission.SETTINGS_VIEW],
  [Role.GUEST]: [],
};

// Check permission
export function hasPermission(role: Role, permission: Permission): boolean {
  return ROLE_PERMISSIONS[role]?.includes(permission) ?? false;
}

// Guard middleware
export function requirePermission(...permissions: Permission[]) {
  return (req: Request, res: Response, next: NextFunction) => {
    const user = req.user;
    if (!user) {
      throw new UnauthorizedError();
    }

    const hasAll = permissions.every(p => hasPermission(user.role, p));
    if (!hasAll) {
      throw new ForbiddenError();
    }

    next();
  };
}

// Usage in routes
app.delete('/posts/:id', authenticate, requirePermission(Permission.POST_DELETE), deletePost);
```

---

## Auth Middleware

```typescript
// src/middleware/auth.ts
import { Request, Response, NextFunction } from 'express';
import { verifyAccessToken } from '../lib/auth.js';
import { UnauthorizedError } from '../utils/errors.js';

export function authenticate(req: Request, res: Response, next: NextFunction) {
  const authHeader = req.headers.authorization;
  if (!authHeader?.startsWith('Bearer ')) {
    throw new UnauthorizedError();
  }

  try {
    const token = authHeader.slice(7);
    const payload = verifyAccessToken(token);
    req.user = {
      id: payload.userId,
      email: payload.email,
      role: payload.role,
    };
    next();
  } catch {
    throw new UnauthorizedError();
  }
}

// Optional auth — doesn't fail if no token
export function maybeAuthenticate(req: Request, res: Response, next: NextFunction) {
  const authHeader = req.headers.authorization;
  if (authHeader?.startsWith('Bearer ')) {
    try {
      const token = authHeader.slice(7);
      req.user = verifyAccessToken(token);
    } catch {
      // Ignore invalid token
    }
  }
  next();
}
```

---

## Security Checklist

- [ ] Passwords hashed with Argon2id (cost 12+) or bcrypt (cost 12+)
- [ ] JWT secret at least 256 bits, stored in env vars
- [ ] HTTPS only in production (secure cookie flag)
- [ ] CSRF protection (sameSite cookie or CSRF token)
- [ ] Rate limiting on auth endpoints (login, register, forgot-password)
- [ ] Account lockout after N failed attempts (5 attempts, 15 min lockout)
- [ ] Password requirements: 8+ chars, not common passwords (zxcvbn check)
- [ ] Refresh token rotation (single-use)
- [ ] Token revocation on logout (store revoked tokens in Redis with TTL)
- [ ] Secure password reset flow (token-based, short expiry 15 min, one-use)
- [ ] OAuth state parameter to prevent CSRF on OAuth redirects
- [ ] No sensitive data in JWT payload (only IDs, roles)
- [ ] Audit log for auth events (login, logout, password change, failed attempts)
