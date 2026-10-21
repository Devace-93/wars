#!/usr/bin/env bash
# Nightly Postgres dump for the wars stack. Keeps 14 days.
#   5 3 * * * /opt/wars/deploy/backup.sh
set -euo pipefail
DIR=/opt/backups/wars
mkdir -p "$DIR"
cd /opt/wars/deploy
docker compose --project-name wars -f docker-compose.yml -f docker-compose.prod.yml exec -T postgres \
  pg_dump -U wars -d wars | gzip > "$DIR/wars-$(date +%F).sql.gz"
find "$DIR" -name 'wars-*.sql.gz' -mtime +14 -delete
