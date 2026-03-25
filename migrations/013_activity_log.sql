-- Activity log: append-only record of significant actions on nodes.
-- subject_id is the Cognito sub of the acting user.
-- metadata is JSONB for optional context (actor display name/email, role, tag name, etc.).
CREATE TABLE activity_log (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id    UUID        NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    subject_id TEXT        NOT NULL,
    action     TEXT        NOT NULL,
    metadata   JSONB       NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX activity_log_node_id_idx ON activity_log(node_id, created_at DESC);
