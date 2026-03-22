-- Sidebar favorites: user-scoped ordered list of pinned nodes or external URLs.
CREATE TABLE user_favorites (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id   TEXT        NOT NULL,
    -- Exactly one of node_id or url must be set (enforced by CHECK below).
    node_id    UUID        REFERENCES nodes(id) ON DELETE CASCADE,
    url        TEXT,
    label      TEXT        NOT NULL CHECK (char_length(label) BETWEEN 1 AND 500),
    position   INT         NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fav_exactly_one_target CHECK (
        (node_id IS NOT NULL AND url IS NULL) OR
        (node_id IS NULL     AND url IS NOT NULL)
    )
);

CREATE INDEX idx_user_favorites_owner ON user_favorites(owner_id, position);

-- Prevent the same node appearing twice in one user's favorites.
CREATE UNIQUE INDEX idx_user_favorites_owner_node
    ON user_favorites(owner_id, node_id) WHERE node_id IS NOT NULL;

-- Prevent the same URL appearing twice in one user's favorites.
CREATE UNIQUE INDEX idx_user_favorites_owner_url
    ON user_favorites(owner_id, url) WHERE url IS NOT NULL;
