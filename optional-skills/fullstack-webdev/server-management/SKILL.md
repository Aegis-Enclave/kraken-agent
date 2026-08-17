---
name: server-management
description: "Production server management — systemd, PM2, Nginx/Caddy reverse proxy, Docker, log rotation, health checks, graceful shutdown."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux]
metadata:
  hermes:
    tags: [server, systemd, pm2, nginx, caddy, deployment, reverse-proxy, production]
    related_skills: [docker-expert, nginx-caddy, github-actions-templates]
---

# Server Management

## Process Managers

### PM2 (recommended for Node.js)

```bash
npm install -g pm2

# Start
pm2 start dist/server.js --name api -i 4  # 4 instances (cluster mode)

# Ecosystem file for complex apps
pm2 start ecosystem.config.js
```

```javascript
// ecosystem.config.js
module.exports = {
  apps: [{
    name: 'api',
    script: 'dist/server.js',
    instances: 4,
    exec_mode: 'cluster',
    env_production: {
      NODE_ENV: 'production',
      PORT: 3000,
    },
    max_memory_restart: '500M',
    restart_delay: 4000,
    max_restarts: 10,
    min_uptime: '10s',
    autorestart: true,
    watch: false,
    ignore_watch: ['node_modules', 'logs'],
    log_file: '/var/log/api.log',
    time: true,
  }],
};
```

```bash
pm2 startup ubuntu     # Generate init script
pm2 save               # Save process list
pm2 resurrect          # Restore on reboot

# Monitoring
pm2 list
pm2 monit
pm2 logs api --lines 50

# Reload without downtime
pm2 reload api
```

### systemd (for any process)

```ini
# /etc/systemd/system/myapp.service
[Unit]
Description=My Web App
After=network.target
Requires=postgresql.service redis.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/myapp
Environment=NODE_ENV=production
ExecStart=/opt/myapp/node_modules/.bin/node dist/server.js
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal
SyslogIdentifier=myapp
# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/myapp/logs

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable myapp
sudo systemctl start myapp
sudo systemctl status myapp
sudo journalctl -u myapp -f
```

---

## Reverse Proxy with Nginx

```nginx
# /etc/nginx/sites-available/api
server {
    listen 80;
    server_name api.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.example.com;

    ssl_certificate /etc/letsencrypt/live/api.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.example.com/privkey.pem;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Gzip
    gzip on;
    gzip_types text/plain application/json application/javascript text/css;
    gzip_min_length 1000;

    # Rate limiting zones
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;

    location / {
        limit_req zone=api burst=20 nodelay;

        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    location /health {
        proxy_pass http://127.0.0.1:3000/health;
        proxy_http_version 1.1;
        # Don't require auth for health checks
    }
}
```

---

## Docker + Production

```yaml
# docker-compose.prod.yml
version: '3.9'
services:
  api:
    image: myregistry/api:latest
    restart: always
    environment:
      - NODE_ENV=production
      - DATABASE_URL=postgres://user:pass@postgres:5432/db
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 128M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"
    networks:
      - web
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started

  postgres:
    image: postgres:16-alpine
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: myapp
      POSTGRES_PASSWORD_FILE: /run/secrets/postgres_password
    secrets:
      - postgres_password
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U myapp -d myapp"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    command: redis-server --maxmemory 256mb --maxmemory-policy allkeys-lru
    volumes:
      - redisdata:/data

networks:
  web:
    driver: bridge

volumes:
  pgdata:
  redisdata:

secrets:
  postgres_password:
    file: ./secrets/postgres_password
```

---

## Log Rotation

```bash
# /etc/logrotate.d/myapp
/var/log/myapp/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 www-data www-data
    sharedscripts
    postrotate
        pm2 reloadLogs 2>/dev/null || true
    endscript
}
```

---

## Health Check Endpoint

Every app should expose `/health`:

```javascript
app.get('/health', async (req, res) => {
  const checks = await Promise.allSettled([
    prisma.$queryRaw`SELECT 1`,
    redis.ping(),
  ]);

  const healthy = checks.every(c => c.status === 'fulfilled');
  res.status(healthy ? 200 : 503).json({
    status: healthy ? 'ok' : 'degraded',
    checks: {
      database: checks[0].status === 'fulfilled' ? 'ok' : 'error',
      redis: checks[1].status === 'fulfilled' ? 'ok' : 'error',
    },
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
  });
});
```

K8s probes:
- `livenessProbe`: `httpGet.path: /health` — is the process alive?
- `readinessProbe`: `httpGet.path: /ready` — can it accept traffic?
- `startupProbe`: for slow-starting apps

---

## Graceful Shutdown

```javascript
const server = app.listen(port, () => {
  logger.info(`Listening on ${port}`);
});

const shutdown = async (signal) => {
  logger.info(`${signal} received, starting graceful shutdown`);

  // Stop accepting new connections
  server.close(async () => {
    logger.info('HTTP server closed');

    // Close DB connections
    await prisma.$disconnect();
    await redis.quit();

    // Stop job queues gracefully
    await emailWorker.close();

    logger.info('All connections closed, exiting');
    process.exit(0);
  });

  // Force exit after timeout
  setTimeout(() => {
    logger.error('Graceful shutdown timeout, forcing exit');
    process.exit(1);
  }, 30000);
};

process.on('SIGTERM', () => shutdown('SIGTERM'));
process.on('SIGINT', () => shutdown('SIGINT'));
```

---

## Environment Variables & Secrets

Never commit secrets to version control.

```bash
# .env (local only, gitignored)
DATABASE_URL=postgres://user:pass@localhost:5432/db
JWT_SECRET=change_me_in_production
REDIS_URL=redis://localhost:6379

# Production: use environment variables or a secrets manager
# AWS: AWS Secrets Manager + Parameter Store
# GCP: Secret Manager
# Docker: docker secret create
```

```javascript
import 'dotenv/config';  // Load .env in development
// Production: variables come from the environment, not .env files
```
