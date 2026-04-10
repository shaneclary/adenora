#!/bin/bash
# Adenora Deployment Script — Hetzner VPS
# Usage: ./scripts/deploy.sh [server-ip]

set -euo pipefail

SERVER=${1:-"adenora-server"}
DEPLOY_DIR="/opt/adenora"

echo "=== Adenora Deploy ==="
echo "Target: $SERVER"

# Build Docker image
echo "Building Docker image..."
docker build -f deploy/Dockerfile -t adenora:latest .

# Build frontend
echo "Building frontend..."
cd web && npm run build && cd ..

# Copy files to server
echo "Deploying to $SERVER..."
ssh "$SERVER" "mkdir -p $DEPLOY_DIR/{deploy,web/build,scripts,backups}"

scp deploy/docker-compose.prod.yml "$SERVER:$DEPLOY_DIR/deploy/"
scp deploy/Caddyfile "$SERVER:$DEPLOY_DIR/deploy/"
scp .env.production "$SERVER:$DEPLOY_DIR/"
scp scripts/seed.sql "$SERVER:$DEPLOY_DIR/scripts/"
rsync -az web/build/ "$SERVER:$DEPLOY_DIR/web/build/"

# Save and load Docker image (alternative to registry)
echo "Transferring Docker image..."
docker save adenora:latest | ssh "$SERVER" "docker load"

# Restart services
echo "Restarting services..."
ssh "$SERVER" "cd $DEPLOY_DIR && docker-compose -f deploy/docker-compose.prod.yml up -d"

# Run migrations (handled by app startup, but verify)
echo "Checking health..."
sleep 5
ssh "$SERVER" "curl -sf http://localhost:8080/healthz || echo 'Health check failed!'"

echo "=== Deploy complete ==="
echo "Check: https://\$(grep ADENORA_DOMAIN $DEPLOY_DIR/.env.production | cut -d= -f2)"
