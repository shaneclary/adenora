#!/bin/bash
# Adenora Database Backup
# Usage: ./scripts/backup.sh
# Add to crontab: 0 3 * * * /opt/adenora/scripts/backup.sh

set -euo pipefail

BACKUP_DIR="/opt/adenora/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
FILENAME="adenora_${TIMESTAMP}.sql.gz"

mkdir -p "$BACKUP_DIR"

# Dump database
docker-compose -f /opt/adenora/deploy/docker-compose.prod.yml exec -T db \
    pg_dump -U "${POSTGRES_USER:-adenora}" "${POSTGRES_DB:-adenora}" \
    | gzip > "$BACKUP_DIR/$FILENAME"

echo "Backup created: $BACKUP_DIR/$FILENAME ($(du -h "$BACKUP_DIR/$FILENAME" | cut -f1))"

# Keep last 30 days of backups
find "$BACKUP_DIR" -name "adenora_*.sql.gz" -mtime +30 -delete
echo "Old backups cleaned up"
