-- Migration 016: Node pinning
--
-- Adds a boolean `pinned` flag to nodes. Pinned nodes always sort to the top
-- of the node list regardless of the selected sort order.

ALTER TABLE nodes ADD COLUMN pinned BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX idx_nodes_pinned ON nodes (pinned) WHERE pinned = TRUE;
