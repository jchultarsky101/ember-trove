#!/usr/bin/env bash
# deploy/backup.sh — automated PostgreSQL backup → S3-compatible object storage
#
# Usage:
#   ./deploy/backup.sh               # run once
#   ./deploy/backup.sh list          # list stored backups
#   ./deploy/backup.sh restore FILE  # restore from a named backup key
#
# Cron (daily at 02:00 UTC):
#   0 2 * * * /home/ubuntu/ember-trove/deploy/backup.sh >> /var/log/ember-backup.log 2>&1
#
# Variables (all can be overridden via .env.prod or environment):
#   BACKUP_BUCKET   S3 bucket for backups (default: same as S3_BUCKET)
#   BACKUP_PREFIX   Key prefix in the bucket  (default: backups)
#   BACKUP_RETAIN   Number of backups to keep  (default: 30)
#   S3_REGION       AWS region                 (default: us-east-2)
#   S3_ENDPOINT     Custom endpoint URL        (optional; for MinIO / Lightsail)
#   POSTGRES_PASSWORD, POSTGRES_DB, POSTGRES_USER, S3_ACCESS_KEY, S3_SECRET_KEY
#     — loaded automatically from deploy/.env.prod if present.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env.prod"

# Load production env file if present (ignored in CI/test environments).
if [[ -f "${ENV_FILE}" ]]; then
    set -a
    # shellcheck disable=SC1090
    source "${ENV_FILE}"
    set +a
fi

# Defaults
POSTGRES_DB="${POSTGRES_DB:-ember_trove}"
POSTGRES_USER="${POSTGRES_USER:-ember_trove}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-}"
S3_SECRET_KEY="${S3_SECRET_KEY:-}"
S3_REGION="${S3_REGION:-us-east-2}"
S3_ENDPOINT="${S3_ENDPOINT:-}"          # empty = native AWS S3; set for MinIO / Lightsail
BACKUP_BUCKET="${BACKUP_BUCKET:-${S3_BUCKET:-}}"
BACKUP_PREFIX="${BACKUP_PREFIX:-backups}"
BACKUP_RETAIN="${BACKUP_RETAIN:-30}"
COMPOSE_DIR="${SCRIPT_DIR}"

# Docker container name for the postgres service
PG_CONTAINER="deploy-postgres-1"

# Ensure required paths are available
export PATH="/usr/local/bin:/Applications/Docker.app/Contents/Resources/bin:${PATH}"

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

# Build AWS CLI environment
aws_env=(
    env
    "AWS_ACCESS_KEY_ID=${S3_ACCESS_KEY}"
    "AWS_SECRET_ACCESS_KEY=${S3_SECRET_KEY}"
    "AWS_DEFAULT_REGION=${S3_REGION}"
)
aws_flags=(--region "${S3_REGION}")
[[ -n "${S3_ENDPOINT}" ]] && aws_flags+=(--endpoint-url "${S3_ENDPOINT}")

aws_cmd() {
    "${aws_env[@]}" aws "${aws_flags[@]}" "$@"
}

# ── Subcommands ──────────────────────────────────────────────────────────────

cmd_list() {
    log "Backups in s3://${BACKUP_BUCKET}/${BACKUP_PREFIX}/"
    aws_cmd s3 ls "s3://${BACKUP_BUCKET}/${BACKUP_PREFIX}/" 2>/dev/null || \
        echo "  (no backups found or bucket not reachable)"
}

cmd_restore() {
    local key="${1:-}"
    if [[ -z "${key}" ]]; then
        echo "Usage: $0 restore <backup-filename>" >&2
        exit 1
    fi
    log "Restoring from s3://${BACKUP_BUCKET}/${BACKUP_PREFIX}/${key} …"
    log "⚠ This will DROP and recreate the ember_trove database!"
    read -rp "Type YES to confirm: " confirm
    [[ "${confirm}" == "YES" ]] || { echo "Aborted."; exit 1; }

    aws_cmd s3 cp "s3://${BACKUP_BUCKET}/${BACKUP_PREFIX}/${key}" - \
        | gunzip \
        | docker exec -i -e "PGPASSWORD=${POSTGRES_PASSWORD}" "${PG_CONTAINER}" \
            psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}"
    log "Restore complete."
}

cmd_backup() {
    if [[ -z "${BACKUP_BUCKET}" ]]; then
        log "ERROR: BACKUP_BUCKET (or S3_BUCKET) is not set." >&2
        exit 1
    fi

    DATETIME=$(date -u +%Y%m%d_%H%M%SZ)
    FILENAME="ember-trove-${DATETIME}.sql.gz"
    S3_KEY="${BACKUP_PREFIX}/${FILENAME}"

    log "Starting backup → s3://${BACKUP_BUCKET}/${S3_KEY}"

    # pg_dump runs inside the postgres container; piped through host gzip → S3.
    docker exec \
        -e "PGPASSWORD=${POSTGRES_PASSWORD}" \
        "${PG_CONTAINER}" \
        pg_dump -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" --no-password \
        | gzip \
        | aws_cmd s3 cp - "s3://${BACKUP_BUCKET}/${S3_KEY}"

    log "Upload complete: ${FILENAME}"

    # ── Prune old backups ────────────────────────────────────────────────────
    TOTAL=$(aws_cmd s3 ls "s3://${BACKUP_BUCKET}/${BACKUP_PREFIX}/" | wc -l)
    if (( TOTAL > BACKUP_RETAIN )); then
        PRUNE=$(( TOTAL - BACKUP_RETAIN ))
        log "Pruning ${PRUNE} old backup(s) (keeping ${BACKUP_RETAIN})…"
        aws_cmd s3 ls "s3://${BACKUP_BUCKET}/${BACKUP_PREFIX}/" \
            | sort \
            | head -n "${PRUNE}" \
            | awk '{print $4}' \
            | while IFS= read -r old_key; do
                aws_cmd s3 rm "s3://${BACKUP_BUCKET}/${BACKUP_PREFIX}/${old_key}"
                log "  Removed: ${old_key}"
              done
    fi

    log "Done. Total backups: ${TOTAL}"
}

# ── Dispatch ─────────────────────────────────────────────────────────────────

case "${1:-backup}" in
    list)    cmd_list ;;
    restore) cmd_restore "${2:-}" ;;
    backup)  cmd_backup ;;
    *)
        echo "Usage: $0 [backup|list|restore <file>]" >&2
        exit 1
        ;;
esac
