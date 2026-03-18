CREATE TABLE IF NOT EXISTS backup_jobs (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by       TEXT        NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    size_bytes       BIGINT      NOT NULL DEFAULT 0,
    s3_key           TEXT        NOT NULL,
    node_count       INT         NOT NULL DEFAULT 0,
    edge_count       INT         NOT NULL DEFAULT 0,
    tag_count        INT         NOT NULL DEFAULT 0,
    note_count       INT         NOT NULL DEFAULT 0,
    task_count       INT         NOT NULL DEFAULT 0,
    attachment_count INT         NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_backup_jobs_created_by ON backup_jobs(created_by);
CREATE INDEX IF NOT EXISTS idx_backup_jobs_created_at ON backup_jobs(created_at DESC);
