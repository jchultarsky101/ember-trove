-- Migration 021: Index on nodes.owner_id
--
-- The owner_id column is the primary multi-tenant access pattern. Without an
-- index, every permission check and node listing triggers a full table scan,
-- which becomes a DoS vector as the nodes table grows.

CREATE INDEX IF NOT EXISTS nodes_owner_id_idx ON nodes(owner_id);
