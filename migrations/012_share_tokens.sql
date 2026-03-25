-- Share tokens: allow owners to generate public read-only links for a node.
-- A token is a random UUID used as an opaque URL slug.
-- Tokens have an optional expiry; NULL means they never expire.
CREATE TABLE share_tokens (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id    UUID        NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    token      UUID        NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    created_by TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX share_tokens_node_id_idx ON share_tokens(node_id);
CREATE INDEX share_tokens_token_idx   ON share_tokens(token);
