-- Prevent duplicate node titles per owner (case-insensitive).
-- Two different owners may share a title; the same owner may not.
CREATE UNIQUE INDEX nodes_owner_title_unique
    ON nodes (owner_id, lower(title));
