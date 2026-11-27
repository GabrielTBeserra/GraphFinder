#!/usr/bin/env bash
set -euo pipefail

out="${1:-backups/graphfinder-$(date -u +%Y%m%dT%H%M%SZ).dump}"
mkdir -p "$(dirname "$out")"

docker compose exec -T postgres pg_dump \
  --username=graphfinder \
  --dbname=graphfinder \
  --format=custom \
  --compress=9 \
  --no-owner \
  --no-privileges > "$out"

printf 'backup written: %s\n' "$out"
