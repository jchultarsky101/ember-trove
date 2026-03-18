-- Append-only notes attached to a node (microblog / decision log).
-- Notes are never edited; only the owner of the node may create them.

CREATE TABLE node_notes (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id    UUID        NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    owner_id   TEXT        NOT NULL,
    body       TEXT        NOT NULL CHECK (char_length(body) BETWEEN 1 AND 10000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX node_notes_node_id_idx    ON node_notes(node_id);
CREATE INDEX node_notes_owner_id_idx   ON node_notes(owner_id);
CREATE INDEX node_notes_created_at_idx ON node_notes(created_at DESC);
