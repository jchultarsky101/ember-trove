-- Migration 007: extend search to cover notes and tasks
--
-- Adds a stored tsvector column to node_notes for full-text search,
-- plus GIN trigram indexes on note body and task title for fuzzy search.
-- pg_trgm is already enabled (migration 001).

ALTER TABLE node_notes
    ADD COLUMN IF NOT EXISTS search_vec tsvector
        GENERATED ALWAYS AS (to_tsvector('english', coalesce(body, ''))) STORED;

CREATE INDEX IF NOT EXISTS idx_node_notes_search
    ON node_notes USING GIN(search_vec);

CREATE INDEX IF NOT EXISTS idx_node_notes_body_trgm
    ON node_notes USING GIN(body gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_node_tasks_title_trgm
    ON node_tasks USING GIN(title gin_trgm_ops);
