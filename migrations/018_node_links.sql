-- Node external links: named URLs attached to a node.
CREATE TABLE node_links (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id    UUID        NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    name       TEXT        NOT NULL CHECK (char_length(name) > 0),
    url        TEXT        NOT NULL CHECK (char_length(url) > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX node_links_node_id_idx ON node_links (node_id);
