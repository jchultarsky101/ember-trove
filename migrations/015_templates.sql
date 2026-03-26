CREATE TABLE node_templates (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT        NOT NULL,
    description TEXT,
    node_type   TEXT        NOT NULL,
    body        TEXT        NOT NULL DEFAULT '',
    created_by  TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX node_templates_created_by_idx ON node_templates(created_by);
