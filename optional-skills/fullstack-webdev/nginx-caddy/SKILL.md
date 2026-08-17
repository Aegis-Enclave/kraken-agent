---
name: nginx-caddy
description: "Nginx and Caddy web server configuration — reverse proxy, SSL/TLS, load balancing, security headers, static files, gzip."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux]
metadata:
  hermes:
    tags: [nginx, caddy, reverse-proxy, ssl, tls, letsencrypt, load-balancing]
    related_skills: [server-management, docker-expert]
---

# Nginx & Caddy

## Nginx Reverse Proxy

```nginx
# /etc/nginx/sites-available/api.example.com

# Rate limiting
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=auth:1m rate=1r/s;

upstream api_backend {
    least_conn;  # Load balancing method
    server 127.0.0.1:3000 weight=5;
    server 127.0.0.1:3001 weight=3;
    server 127.0.0.1:3002 weight=2;
    keepalive 32;
}

server {
    listen 80;
    listen [::]:80;
    server_name api.example.com;

    # Let's Encrypt
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://$host$request_uri;
    }
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name api.example.com;

    ssl_certificate /etc/letsencrypt/live/api.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.example.com/privkey.pem;
    ssl_trusted_certificate /etc/letsencrypt/live/api.example.com/chain.pem;

    # Modern TLS
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Permissions-Policy "camera=(), microphone=(), geolocation=()" always;
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;

    # Gzip
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript
               application/xml application/xml+rss text/javascript application/x-javascript;

    client_max_body_size 10M;

    # Proxy to Node.js app
    location / {
        limit_req zone=api burst=20 nodelay;

        proxy_pass http://api_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Request-ID $request_id;
        proxy_cache_bypass $http_upgrade;

        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check (no rate limit, no auth)
    location /health {
        proxy_pass http://api_backend/health;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
    }

    # Static files
    location /static/ {
        alias /var/www/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
        access_log off;
    }
}
```

---

## Nginx Load Balancer

```nginx
# /etc/nginx/nginx.conf
worker_processes auto;
worker_rlimit_nofile 65535;

events {
    worker_connections 4096;
    multi_accept on;
    use epoll;
}

http {
    # Buffer sizes
    client_body_buffer_size 16k;
    client_header_buffer_size 1k;
    client_max_body_size 8m;
    large_client_header_buffers 4 8k;

    upstream backend {
        least_conn;
        server 10.0.1.10:3000 max_fails=3 fail_timeout=30s;
        server 10.0.1.11:3000 max_fails=3 fail_timeout=30s;
        server 10.0.1.12:3000 max_fails=3 fail_timeout=30s;
        keepalive 64;
    }

    # Simple round-robin
    upstream api {
        server 10.0.1.10:3000;
        server 10.0.1.11:3000;
        server 10.0.1.12:3000;
    }
}
```

---

## Let's Encrypt with Certbot

```bash
# Install
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d api.example.com -d www.api.example.com

# Auto-renewal (already set up by certbot)
sudo certbot renew --dry-run

# Or manually
sudo certbot renew --nginx
```

```bash
# Cron job (auto-renewal)
sudo crontab -e
# Add: 0 0 * * * certbot renew --quiet --deploy-hook "systemctl reload nginx"
```

---

## Caddy (Simple Alternative)

Caddy automatically handles HTTPS and offers a much simpler config:

```bash
# Install
sudo apt install -y apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update && sudo apt install caddy
```

```caddy
# Caddyfile
api.example.com {
    reverse_proxy localhost:3000

    # Rate limiting
    @auth {
        path /api/auth/*
    }
    limit @auth {
        rate 5
        period 1m
    }

    # PHP (if needed)
    php_fastcgi localhost:9000

    # Static files
    handle /static/* {
        root * /var/www/static
        file_server
        encode gzip
        cache {
            match_path /static/*
        }
    }

    # Security headers
    header {
        X-Frame-Options "SAMEORIGIN"
        X-Content-Type-Options "nosniff"
        X-XSS-Protection "1; mode=block"
        Referrer-Policy "strict-origin-when-cross-origin"
        Permissions-Policy "camera=(), microphone=(), geolocation=()"
        Strict-Transport-Security "max-age=63072000; includeSubDomains; preload"
    }

    log {
        output file /var/log/caddy/api.example.com.log
    }
}
```

---

## Brotli Compression (Nginx)

```nginx
# Add to http block
load_module modules/ngx_http_brotli_filter_module.so;
load_module modules/ngx_http_brotli_static_module.so;

http {
    brotli on;
    brotli_types text/plain text/css text/xml application/json application/javascript
                application/xml application/xml+rss text/javascript;
    brotli_comp_level 6;
    brotli_static on;
}
```

---

## WordPress/Nginx

```nginx
location / {
    try_files $uri $uri/ /index.php?$args;
}

location ~ \.php$ {
    fastcgi_pass unix:/var/run/php/php-fpm.sock;
    fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
    include fastcgi_params;
    fastcgi_read_timeout 300;
}

location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
    expires 30d;
    add_header Cache-Control "public, immutable";
}

location ~ /\. {
    deny all;
}
```

---

## Troubleshooting

```bash
# Test config
sudo nginx -t

# Reload after config change
sudo systemctl reload nginx

# Restart on crash
sudo systemctl restart nginx

# View logs
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log

# Check what's listening
sudo ss -tlnp | grep :80
```
