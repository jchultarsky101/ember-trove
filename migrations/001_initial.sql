-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- ── Enums ──────────────────────────────────────────────────────────────────

CREATE TYPE node_type AS ENUM (
    'article',
    'project',
    'area',
    'resource',
    'reference'
);

CREATE TYPE node_status AS ENUM (
    'draft',
    'published',
    'archived'
);

CREATE TYPE edge_type AS ENUM (
    'references',
    'contains',
    'related_to',
    'depends_on',
    'derived_from'
);

CREATE TYPE permission_role AS ENUM (
    'owner',
    'editor',
    'viewer'
);

-- ── nodes ──────────────────────────────────────────────────────────────────

CREATE TABLE nodes (
    id         UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id   TEXT        NOT NULL,
    node_type  node_type   NOT NULL,
    title      TEXT        NOT NULL,
    slug       TEXT        NOT NULL UNIQUE,
    body       TEXT,
    metadata   JSONB       NOT NULL DEFAULT '{}',
    status     node_status NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Full-text search vector (generated, stored)
ALTER TABLE nodes
    ADD COLUMN search_vec tsvector
        GENERATED ALWAYS AS (
            setweight(to_tsvector('english', coalesce(title, '')), 'A') ||
            setweight(to_tsvector('english', coalesce(body,  '')), 'B')
        ) STORED;

CREATE INDEX idx_nodes_search  ON nodes USING GIN(search_vec);
CREATE INDEX idx_nodes_owner   ON nodes(owner_id);
CREATE INDEX idx_nodes_type    ON nodes(node_type);
CREATE INDEX idx_nodes_status  ON nodes(status);
CREATE INDEX idx_nodes_slug    ON nodes(slug);

-- Fuzzy search on title
CREATE INDEX idx_nodes_title_trgm ON nodes USING GIN(title gin_trgm_ops);

-- ── edges ──────────────────────────────────────────────────────────────────

CREATE TABLE edges (
    id         UUID      PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_id  UUID      NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    target_id  UUID      NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    edge_type  edge_type NOT NULL,
    label      TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT edges_no_self_loop CHECK (source_id <> target_id)
);

CREATE INDEX idx_edges_source ON edges(source_id);
CREATE INDEX idx_edges_target ON edges(target_id);
CREATE INDEX idx_edges_type   ON edges(edge_type);

-- ── tags ───────────────────────────────────────────────────────────────────

CREATE TABLE tags (
    id         UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id   TEXT        NOT NULL,
    name       TEXT        NOT NULL,
    color      TEXT        NOT NULL DEFAULT '#3b82f6',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner_id, name)
);

CREATE INDEX idx_tags_owner ON tags(owner_id);

-- ── node_tags ──────────────────────────────────────────────────────────────

CREATE TABLE node_tags (
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    tag_id  UUID NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (node_id, tag_id)
);

CREATE INDEX idx_node_tags_tag ON node_tags(tag_id);

-- ── attachments ────────────────────────────────────────────────────────────

CREATE TABLE attachments (
    id           UUID   PRIMARY KEY DEFAULT uuid_generate_v4(),
    node_id      UUID   NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    filename     TEXT   NOT NULL,
    content_type TEXT   NOT NULL DEFAULT 'application/octet-stream',
    size_bytes   BIGINT NOT NULL DEFAULT 0,
    s3_key       TEXT   NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attachments_node ON attachments(node_id);

-- ── permissions ────────────────────────────────────────────────────────────

CREATE TABLE permissions (
    id         UUID            PRIMARY KEY DEFAULT uuid_generate_v4(),
    node_id    UUID            NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    subject_id TEXT            NOT NULL,
    role       permission_role NOT NULL,
    granted_by TEXT            NOT NULL,
    created_at TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    UNIQUE(node_id, subject_id)
);

CREATE INDEX idx_permissions_node    ON permissions(node_id);
CREATE INDEX idx_permissions_subject ON permissions(subject_id);

-- ── updated_at trigger ─────────────────────────────────────────────────────

CREATE OR REPLACE FUNCTION touch_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER nodes_updated_at
    BEFORE UPDATE ON nodes
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
