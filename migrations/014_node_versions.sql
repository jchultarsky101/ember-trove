-- Node body version history
-- A snapshot is recorded automatically on every successful update_node call.
-- Versions are append-only; restoring creates a new edit (and a new snapshot).

CREATE TABLE node_versions (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id     UUID        NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    body        TEXT        NOT NULL,
    created_by  TEXT        NOT NULL,   -- Cognito sub of the saving user
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX node_versions_node_id_idx ON node_versions(node_id, created_at DESC);
