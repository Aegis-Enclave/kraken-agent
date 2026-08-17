---
name: docker-expert
description: "Docker containerization for full-stack web apps — multi-stage builds, layer caching, Docker Compose dev/prod, security hardening."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos]
metadata:
  hermes:
    tags: [docker, containerization, multi-stage-build, docker-compose, devops, production]
    related_skills: [github-actions-templates, nodejs-backend-patterns, server-management]
---

# Docker Expert

## Node.js Dockerfile (multi-stage)

```dockerfile
# syntax=docker/dockerfile:1

# ─── Build stage ───
FROM node:20-alpine AS builder
WORKDIR /app

# Install deps first (layer caching: deps change less often)
COPY package*.json ./
RUN npm ci --only=production=false

COPY . .
RUN npm run build

# ─── Production stage ───
FROM node:20-alpine AS production
WORKDIR /app

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodeapp -u 1001

# Copy built artifacts
COPY --from=builder --chown=nodeapp:nodejs /app/dist ./dist
COPY --from=builder --chown=nodeapp:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=nodeapp:nodejs /app/package.json ./

USER nodeapp

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD wget -qO- http://localhost:3000/health || exit 1

CMD ["node", "dist/server.js"]
```

---

## Python/FastAPI Dockerfile

```dockerfile
FROM python:3.12-slim AS builder
WORKDIR /app

# Install uv for faster installs
RUN pip install uv

COPY requirements.txt .
RUN uv pip install --system --no-cache -r requirements.txt

COPY . .
RUN uv pip install --system --no-cache -e .

FROM python:3.12-slim AS production
WORKDIR /app

RUN groupadd -r appuser && useradd -r -g appuser appuser

COPY --from=builder --chown=appuser:appuser /app ./app
COPY --from=builder --chown=appuser:appuser /root/.local /root/.local

ENV PATH=/root/.local/bin:$PATH
USER appuser

EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=5s CMD curl -f http://localhost:8000/health || exit 1

CMD ["fastapi", "run", "app/main.py", "--host", "0.0.0.0", "--port", "8000"]
```

---

## Docker Compose — Development

```yaml
version: '3.9'
services:
  api:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "3000:3000"
    volumes:
      - .:/app
      - /app/node_modules
    environment:
      - NODE_ENV=development
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/devdb
      - REDIS_URL=redis://redis:6379
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
    command: npm run dev

  postgres:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: devdb
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    volumes:
      - postgres_dev:/var/lib/postgresql/data
      - ./db/init.sql:/docker-entrypoint-initdb.d/init.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_dev:/data

  mailhog:
    image: mailhog/mailhog:latest
    ports:
      - "1025:1025"  # SMTP server
      - "8025:8025"  # Web UI

volumes:
  postgres_dev:
  redis_dev:
```

```dockerfile
# Dockerfile.dev (separate from production)
FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
CMD ["npm", "run", "dev"]
```

---

## .dockerignore

```
node_modules
.git
.env*
dist
*.log
.DS_Store
coverage
*.test.js
.vscode
.idea
__pycache__
*.pyc
.env
docker-compose*.yml
Dockerfile*
README.md
```

---

## Production Hardening

```dockerfile
# Add at end of production Dockerfile
# Read-only filesystem
USER root
RUN chmod 555 /app

# Drop all capabilities
USER nodeapp

# Additional security
ENV NODE_ENV=production
```

docker-compose.yml additions:
```yaml
security_opt:
  - no-new-privileges:true
read_only: true
tmpfs:
  - /tmp
```

---

## Buildx Multi-Platform

```bash
# Setup
docker buildx create --name mybuilder --driver docker-container --use
docker buildx inspect --bootstrap

# Build for multiple platforms
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --tag myregistry/api:latest \
  --push \
  --cache-from type=registry,ref=myregistry/api:buildcache \
  --cache-to type=registry,ref=myregistry/api:buildcache,mode=max \
  .
```

---

## Layer Caching in CI

```yaml
# GitHub Actions
- name: Build Docker image
  uses: docker/build-push-action@v5
  with:
    context: .
    push: false
    load: true
    tags: myapp:${{ github.sha }}
    cache-from: type=gha
    cache-to: type=gha,mode=max
```

---

## Docker Inspect & Debug

```bash
docker ps                          # Running containers
docker logs -f <container>         # Follow logs
docker exec -it <container> sh     # Shell into container
docker inspect <container>         # Full config
docker stats                       # Resource usage
docker network inspect <network>   # Network details
docker volume ls
docker system df                    # Disk usage
docker compose ps
docker compose logs -f api
```
