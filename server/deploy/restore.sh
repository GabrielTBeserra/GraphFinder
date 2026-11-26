#!/usr/bin/env bash
set -euo pipefail

backup_file="${1:-}"
if [[ -z "$backup_file" || ! -f "$backup_file" ]]; then
  printf 'usage: %s <backup-file.dump>\n' "$0" >&2
  exit 2
fi

printf 'This restores into the compose PostgreSQL database and may overwrite existing data.\n' >&2
printf 'Set GF_CONFIRM_RESTORE=yes to continue.\n' >&2
if [[ "${GF_CONFIRM_RESTORE:-}" != "yes" ]]; then
  exit 2
fi

docker compose exec -T postgres pg_restore \
  --username=graphfinder \
  --dbname=graphfinder \
  --clean \
  --if-exists \
  --no-owner \
  --no-privileges < "$backup_file"

printf 'restore completed from: %s\n' "$backup_file"
