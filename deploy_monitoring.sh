#!/bin/bash

# Monitoring Stack Deployment Script
# Deploys Prometheus, Grafana, Loki, and Alertmanager

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}HackerExperience Monitoring Stack Deployment${NC}"
echo "============================================="
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Docker is not installed. Please install Docker first.${NC}"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo -e "${YELLOW}docker-compose not found, trying docker compose...${NC}"
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

# Create necessary directories
echo -e "${YELLOW}Creating monitoring directories...${NC}"
mkdir -p monitoring/prometheus
mkdir -p monitoring/grafana/dashboards
mkdir -p monitoring/grafana/provisioning/dashboards
mkdir -p monitoring/grafana/provisioning/datasources
mkdir -p monitoring/loki
mkdir -p monitoring/promtail
mkdir -p monitoring/alertmanager

# Create Prometheus configuration
echo -e "${YELLOW}Creating Prometheus configuration...${NC}"
cat > monitoring/prometheus/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - alertmanager:9093

rule_files:
  - /etc/prometheus/rules/*.yml

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'hackerexperience'
    static_configs:
      - targets: ['api:3005']
    metrics_path: '/metrics'

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  - job_name: 'loki'
    static_configs:
      - targets: ['loki:3100']
EOF

# Create Loki configuration
echo -e "${YELLOW}Creating Loki configuration...${NC}"
cat > monitoring/loki/loki-config.yaml << 'EOF'
auth_enabled: false

server:
  http_listen_port: 3100
  grpc_listen_port: 9096

common:
  path_prefix: /tmp/loki
  storage:
    filesystem:
      chunks_directory: /tmp/loki/chunks
      rules_directory: /tmp/loki/rules
  replication_factor: 1
  ring:
    instance_addr: 127.0.0.1
    kvstore:
      store: inmemory

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

ruler:
  alertmanager_url: http://alertmanager:9093

analytics:
  reporting_enabled: false
EOF

# Create Promtail configuration
echo -e "${YELLOW}Creating Promtail configuration...${NC}"
cat > monitoring/promtail/promtail-config.yaml << 'EOF'
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: system
    static_configs:
      - targets:
          - localhost
        labels:
          job: varlogs
          __path__: /var/log/*log

  - job_name: hackerexperience
    static_configs:
      - targets:
          - localhost
        labels:
          job: hackerexperience
          __path__: /app/logs/*.log

  - job_name: docker
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: ['__meta_docker_container_name']
        regex: '/(.*)'
        target_label: 'container'
EOF

# Create Grafana datasource configuration
echo -e "${YELLOW}Creating Grafana datasource configuration...${NC}"
cat > monitoring/grafana/provisioning/datasources/datasources.yaml << 'EOF'
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true

  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
    editable: true

  - name: PostgreSQL
    type: postgres
    url: postgres:5432
    database: hackerexperience
    user: $POSTGRES_USER
    secureJsonData:
      password: $POSTGRES_PASSWORD
    jsonData:
      sslmode: disable
    editable: true
EOF

# Create Grafana dashboard configuration
echo -e "${YELLOW}Creating Grafana dashboard provisioning...${NC}"
cat > monitoring/grafana/provisioning/dashboards/dashboards.yaml << 'EOF'
apiVersion: 1

providers:
  - name: 'HackerExperience'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards
EOF

# Create a sample dashboard
echo -e "${YELLOW}Creating sample Grafana dashboard...${NC}"
cat > monitoring/grafana/provisioning/dashboards/hackerexperience.json << 'EOF'
{
  "dashboard": {
    "id": null,
    "uid": "hackerexp-main",
    "title": "HackerExperience Main Dashboard",
    "timezone": "browser",
    "schemaVersion": 16,
    "version": 1,
    "panels": [
      {
        "gridPos": { "h": 8, "w": 12, "x": 0, "y": 0 },
        "id": 1,
        "title": "Online Players",
        "type": "graph",
        "targets": [
          {
            "expr": "game_online_players",
            "refId": "A"
          }
        ]
      },
      {
        "gridPos": { "h": 8, "w": 12, "x": 12, "y": 0 },
        "id": 2,
        "title": "API Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "refId": "A"
          }
        ]
      }
    ]
  }
}
EOF

# Copy alert rules to Prometheus
echo -e "${YELLOW}Setting up alert rules...${NC}"
if [ -f monitoring/alerts/rules.yml ]; then
    cp monitoring/alerts/rules.yml monitoring/prometheus/rules.yml
    echo -e "${GREEN}✓ Alert rules copied${NC}"
fi

# Copy alertmanager config
if [ -f monitoring/alertmanager.yml ]; then
    cp monitoring/alertmanager.yml monitoring/alertmanager/config.yml
    echo -e "${GREEN}✓ Alertmanager config copied${NC}"
fi

# Create monitoring docker-compose override
echo -e "${YELLOW}Creating enhanced monitoring docker-compose...${NC}"
cat > docker-compose.monitoring-full.yml << 'EOF'
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    restart: unless-stopped
    volumes:
      - ./monitoring/prometheus:/etc/prometheus
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--web.enable-lifecycle'
    ports:
      - "9090:9090"
    networks:
      - monitoring

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=hackerexp2024
      - GF_INSTALL_PLUGINS=redis-datasource,grafana-piechart-panel
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning
    ports:
      - "3000:3000"
    networks:
      - monitoring

  loki:
    image: grafana/loki:latest
    container_name: loki
    restart: unless-stopped
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki/loki-config.yaml:/etc/loki/local-config.yaml
      - loki_data:/loki
    command: -config.file=/etc/loki/local-config.yaml
    networks:
      - monitoring

  promtail:
    image: grafana/promtail:latest
    container_name: promtail
    restart: unless-stopped
    volumes:
      - /var/log:/var/log:ro
      - ./logs:/app/logs:ro
      - ./monitoring/promtail/promtail-config.yaml:/etc/promtail/config.yml
      - /var/run/docker.sock:/var/run/docker.sock:ro
    command: -config.file=/etc/promtail/config.yml
    networks:
      - monitoring

  alertmanager:
    image: prom/alertmanager:latest
    container_name: alertmanager
    restart: unless-stopped
    ports:
      - "9093:9093"
    volumes:
      - ./monitoring/alertmanager:/etc/alertmanager
      - alertmanager_data:/alertmanager
    command:
      - '--config.file=/etc/alertmanager/config.yml'
      - '--storage.path=/alertmanager'
    networks:
      - monitoring

  node-exporter:
    image: prom/node-exporter:latest
    container_name: node-exporter
    restart: unless-stopped
    ports:
      - "9100:9100"
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    command:
      - '--path.procfs=/host/proc'
      - '--path.rootfs=/rootfs'
      - '--path.sysfs=/host/sys'
      - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
    networks:
      - monitoring

  postgres-exporter:
    image: prometheuscommunity/postgres-exporter:latest
    container_name: postgres-exporter
    restart: unless-stopped
    environment:
      DATA_SOURCE_NAME: "postgresql://hackerexp:password@postgres:5432/hackerexperience?sslmode=disable"
    ports:
      - "9187:9187"
    networks:
      - monitoring

  redis-exporter:
    image: oliver006/redis_exporter:latest
    container_name: redis-exporter
    restart: unless-stopped
    environment:
      REDIS_ADDR: "redis:6379"
    ports:
      - "9121:9121"
    networks:
      - monitoring

volumes:
  prometheus_data:
  grafana_data:
  loki_data:
  alertmanager_data:

networks:
  monitoring:
    driver: bridge
EOF

# Deploy the stack
echo ""
echo -e "${GREEN}Deploying monitoring stack...${NC}"
echo ""

if $DOCKER_COMPOSE -f docker-compose.monitoring-full.yml up -d; then
    echo -e "${GREEN}✓ Monitoring stack deployed successfully!${NC}"
    echo ""
    echo "Access points:"
    echo "  - Prometheus: http://localhost:9090"
    echo "  - Grafana: http://localhost:3000 (admin/hackerexp2024)"
    echo "  - Alertmanager: http://localhost:9093"
    echo "  - Loki: http://localhost:3100"
    echo ""

    # Wait for services to be ready
    echo -e "${YELLOW}Waiting for services to be ready...${NC}"
    sleep 10

    # Check service health
    echo -e "${YELLOW}Checking service health...${NC}"

    if curl -f http://localhost:9090/-/healthy > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Prometheus is healthy${NC}"
    else
        echo -e "${RED}✗ Prometheus is not responding${NC}"
    fi

    if curl -f http://localhost:3000/api/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Grafana is healthy${NC}"
    else
        echo -e "${YELLOW}⚠ Grafana is still starting up${NC}"
    fi

    if curl -f http://localhost:3100/ready > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Loki is ready${NC}"
    else
        echo -e "${YELLOW}⚠ Loki is still starting up${NC}"
    fi

    echo ""
    echo -e "${GREEN}Monitoring stack deployment complete!${NC}"
else
    echo -e "${RED}Failed to deploy monitoring stack${NC}"
    exit 1
fi