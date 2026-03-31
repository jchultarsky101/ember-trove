-- Add colour key to notes. Stored as a named palette key (e.g. "amber", "rose").
-- "default" means no background colour (neutral stone card).
ALTER TABLE node_notes
    ADD COLUMN color TEXT NOT NULL DEFAULT 'default';
