# HackerExperience Production Deployment Guide

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Server Requirements](#server-requirements)
3. [Quick Deploy with Docker](#quick-deploy-with-docker)
4. [Manual Deployment](#manual-deployment)
5. [SSL/TLS Setup](#ssltls-setup)
6. [Domain Configuration](#domain-configuration)
7. [Monitoring Setup](#monitoring-setup)
8. [Backup & Recovery](#backup--recovery)
9. [Security Checklist](#security-checklist)
10. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Software
- Docker 24.0+ & Docker Compose 2.20+
- PostgreSQL 16+
- Redis 7+
- Nginx 1.24+
- Git 2.40+
- Certbot (for SSL certificates)

### Recommended OS
- Ubuntu 22.04 LTS
- Debian 12
- RHEL 9

---

## Server Requirements

### Minimum (100 concurrent users)
- **CPU**: 4 cores @ 2.4GHz
- **RAM**: 8GB
- **Storage**: 50GB SSD
- **Bandwidth**: 100Mbps

### Recommended (1000 concurrent users)
- **CPU**: 8 cores @ 3.0GHz
- **RAM**: 16GB
- **Storage**: 100GB NVMe SSD
- **Bandwidth**: 1Gbps

### Production (10,000+ concurrent users)
- **CPU**: 16+ cores @ 3.5GHz
- **RAM**: 32GB+
- **Storage**: 500GB NVMe SSD RAID
- **Bandwidth**: 10Gbps
- **Load Balancer**: Required
- **CDN**: Recommended

---

## Quick Deploy with Docker

### 1. Clone Repository
```bash
git clone https://github.com/yourusername/hackerexperience-rust.git
cd hackerexperience-rust
```

### 2. Configure Environment
```bash
cp .env.example .env
nano .env
```

Required environment variables:
```env
# Database
DB_USER=heuser
DB_PASSWORD=<strong_password>

# Redis
REDIS_PASSWORD=<strong_password>

# Application
JWT_SECRET=<generate_with_openssl_rand_-hex_32>
ENCRYPTION_KEY=<generate_with_openssl_rand_-hex_32>

# Monitoring
GRAFANA_PASSWORD=<admin_password>

# Domain
DOMAIN=hackerexperience.com
```

### 3. Deploy with Docker Compose
```bash
# Production deployment
docker-compose -f docker-compose.production.yml up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f app
```

### 4. Run Database Migrations
```bash
docker-compose exec app sqlx migrate run
```

### 5. Verify Deployment
```bash
# Health check
curl http://localhost:3000/health

# API test
curl http://localhost:3000/api/status
```

---

## Manual Deployment

### 1. Install Dependencies
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install PostgreSQL
sudo apt install postgresql postgresql-contrib -y

# Install Redis
sudo apt install redis-server -y

# Install Nginx
sudo apt install nginx -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Setup Database
```bash
# Create database and user
sudo -u postgres psql <<EOF
CREATE DATABASE hackerexperience;
CREATE USER heuser WITH ENCRYPTED PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE hackerexperience TO heuser;
EOF

# Run migrations
export DATABASE_URL="postgresql://heuser:your_password@localhost/hackerexperience"
sqlx migrate run
```

### 3. Build Application
```bash
# Build in release mode
cargo build --release

# Copy binary
sudo cp target/release/he-api /usr/local/bin/
```

### 4. Create Systemd Service
```bash
sudo nano /etc/systemd/system/hackerexperience.service
```

```ini
[Unit]
Description=HackerExperience Game Server
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/hackerexperience
Environment="DATABASE_URL=postgresql://heuser:password@localhost/hackerexperience"
Environment="REDIS_URL=redis://localhost:6379"
Environment="JWT_SECRET=your_secret"
Environment="PORT=3000"
ExecStart=/usr/local/bin/he-api
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### 5. Start Service
```bash
sudo systemctl enable hackerexperience
sudo systemctl start hackerexperience
sudo systemctl status hackerexperience
```

---

## SSL/TLS Setup

### Using Certbot (Let's Encrypt)
```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx -y

# Generate certificate
sudo certbot --nginx -d hackerexperience.com -d www.hackerexperience.com

# Auto-renewal
sudo certbot renew --dry-run
```

### Manual SSL Configuration
```bash
sudo nano /etc/nginx/sites-available/hackerexperience
```

```nginx
server {
    listen 80;
    server_name hackerexperience.com www.hackerexperience.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name hackerexperience.com;

    ssl_certificate /etc/letsencrypt/live/hackerexperience.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/hackerexperience.com/privkey.pem;

    # SSL Security Settings
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Proxy to application
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # Static files
    location /assets {
        alias /opt/hackerexperience/frontend/assets;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
}
```

### Enable Site
```bash
sudo ln -s /etc/nginx/sites-available/hackerexperience /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## Domain Configuration

### DNS Records
Add these DNS records to your domain:

```
Type    Name    Value                   TTL
A       @       YOUR_SERVER_IP          300
A       www     YOUR_SERVER_IP          300
A       api     YOUR_SERVER_IP          300
AAAA    @       YOUR_SERVER_IPV6        300 (optional)
CAA     @       0 issue "letsencrypt.org" 300
```

### Cloudflare Configuration (Optional)
1. Add site to Cloudflare
2. Set SSL/TLS mode to "Full (strict)"
3. Enable "Always Use HTTPS"
4. Configure Page Rules:
   - `api.hackerexperience.com/*` - Cache Level: Bypass
   - `*.hackerexperience.com/assets/*` - Cache Level: Cache Everything

---

## Monitoring Setup

### Prometheus Configuration
```yaml
# /opt/hackerexperience/monitoring/prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'hackerexperience'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'

  - job_name: 'node'
    static_configs:
      - targets: ['localhost:9100']

  - job_name: 'postgres'
    static_configs:
      - targets: ['localhost:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['localhost:9121']
```

### Grafana Dashboards
Import these dashboard IDs:
- **Application**: 14531
- **PostgreSQL**: 9628
- **Redis**: 11835
- **Node Exporter**: 1860

Access Grafana at: `http://your-server:3001`
Default login: `admin` / `your_configured_password`

### Health Monitoring Script
```bash
#!/bin/bash
# /opt/hackerexperience/scripts/health_check.sh

API_URL="https://hackerexperience.com/health"
WEBHOOK_URL="your_slack_webhook_url"

response=$(curl -s -o /dev/null -w "%{http_code}" $API_URL)

if [ $response -ne 200 ]; then
    curl -X POST $WEBHOOK_URL -H 'Content-Type: application/json' \
        -d '{"text":"⚠️ HackerExperience is down! HTTP Status: '$response'"}'
fi
```

Add to crontab:
```bash
*/5 * * * * /opt/hackerexperience/scripts/health_check.sh
```

---

## Backup & Recovery

### Automated Backup Script
```bash
#!/bin/bash
# /opt/hackerexperience/scripts/backup.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/hackerexperience"

# Database backup
pg_dump hackerexperience | gzip > $BACKUP_DIR/db_$DATE.sql.gz

# Redis backup
redis-cli --rdb $BACKUP_DIR/redis_$DATE.rdb

# Application files
tar -czf $BACKUP_DIR/app_$DATE.tar.gz /opt/hackerexperience

# Keep only last 30 days
find $BACKUP_DIR -name "*.gz" -mtime +30 -delete
find $BACKUP_DIR -name "*.rdb" -mtime +30 -delete

# Upload to S3 (optional)
aws s3 sync $BACKUP_DIR s3://your-backup-bucket/hackerexperience/
```

### Restore Procedure
```bash
# Stop services
sudo systemctl stop hackerexperience

# Restore database
gunzip < backup.sql.gz | psql hackerexperience

# Restore Redis
redis-cli --rdb /path/to/backup.rdb

# Start services
sudo systemctl start hackerexperience
```

---

## Security Checklist

### System Security
- [ ] Firewall configured (UFW/iptables)
- [ ] SSH key-only authentication
- [ ] Fail2ban installed and configured
- [ ] Regular security updates enabled
- [ ] SELinux/AppArmor enabled

### Application Security
- [ ] Environment variables secured
- [ ] Database passwords strong (20+ characters)
- [ ] JWT secret rotated regularly
- [ ] Rate limiting enabled
- [ ] CORS properly configured
- [ ] Input validation active
- [ ] SQL injection protection verified
- [ ] XSS protection enabled

### Network Security
- [ ] SSL/TLS certificates valid
- [ ] HTTPS redirect enabled
- [ ] Security headers configured
- [ ] DDoS protection active
- [ ] WAF configured (if using Cloudflare)

### Monitoring
- [ ] Application metrics enabled
- [ ] Error tracking configured
- [ ] Uptime monitoring active
- [ ] Log aggregation setup
- [ ] Alerting configured

---

## Troubleshooting

### Application Won't Start
```bash
# Check logs
journalctl -u hackerexperience -n 100

# Verify database connection
psql -h localhost -U heuser -d hackerexperience -c "SELECT 1"

# Check Redis
redis-cli ping

# Verify port availability
sudo lsof -i :3000
```

### High Memory Usage
```bash
# Check memory usage
free -h
ps aux | grep he-api

# Restart service
sudo systemctl restart hackerexperience

# Clear Redis cache if needed
redis-cli FLUSHDB
```

### Database Performance Issues
```bash
# Check slow queries
psql -U heuser -d hackerexperience -c "
SELECT query, calls, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;"

# Run VACUUM
psql -U heuser -d hackerexperience -c "VACUUM ANALYZE;"

# Check index usage
psql -U heuser -d hackerexperience -c "
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
ORDER BY idx_scan;"
```

### SSL Certificate Issues
```bash
# Test SSL
openssl s_client -connect hackerexperience.com:443

# Renew certificate
sudo certbot renew

# Restart Nginx
sudo systemctl restart nginx
```

---

## Support

- **Documentation**: https://github.com/yourusername/hackerexperience-rust/wiki
- **Issues**: https://github.com/yourusername/hackerexperience-rust/issues
- **Discord**: https://discord.gg/hackerexperience
- **Email**: support@hackerexperience.com

---

## License

MIT License - See LICENSE file for details.