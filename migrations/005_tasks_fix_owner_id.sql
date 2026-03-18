-- Fix owner_id type: should be TEXT (matching nodes/tags convention) not UUID.
-- Keycloak subjects are UUID-formatted strings but stored as TEXT elsewhere.
ALTER TABLE node_tasks ALTER COLUMN owner_id TYPE TEXT USING owner_id::text;
ALTER INDEX node_tasks_owner_id_idx RENAME TO node_tasks_owner_id_idx_old;
DROP INDEX node_tasks_owner_id_idx_old;
CREATE INDEX node_tasks_owner_id_idx ON node_tasks(owner_id);
DROP INDEX node_tasks_focus_date_idx;
CREATE INDEX node_tasks_focus_date_idx ON node_tasks(owner_id, focus_date)
    WHERE focus_date IS NOT NULL;
