-- Add is_default flag to node_templates.
-- A partial unique index enforces that each owner can have at most one default
-- per node type.  Setting a new default automatically clears the old one in the
-- application layer (set_default uses a transaction), but the index provides a
-- safety net at the DB level.
ALTER TABLE node_templates
    ADD COLUMN is_default BOOLEAN NOT NULL DEFAULT FALSE;

CREATE UNIQUE INDEX node_templates_default_uidx
    ON node_templates (created_by, node_type)
    WHERE is_default = TRUE;
