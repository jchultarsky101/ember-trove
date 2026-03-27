-- Search presets: saved combinations of search query + filters.
CREATE TABLE search_presets (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id       TEXT        NOT NULL,
    name           TEXT        NOT NULL,
    query          TEXT        NOT NULL DEFAULT '',
    fuzzy          BOOLEAN     NOT NULL DEFAULT FALSE,
    published_only BOOLEAN     NOT NULL DEFAULT FALSE,
    -- Comma-separated UUID strings (mirrors SearchQuery::tag_ids format).
    tag_ids        TEXT        NOT NULL DEFAULT '',
    -- 'or' or 'and'
    tag_op         TEXT        NOT NULL DEFAULT 'or',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_search_presets_owner ON search_presets (owner_id);
