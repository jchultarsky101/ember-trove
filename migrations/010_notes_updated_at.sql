-- Allow notes to be edited: add updated_at column and an auto-update trigger.
ALTER TABLE node_notes
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Backfill: existing notes set their updated_at to match created_at.
UPDATE node_notes SET updated_at = created_at;

CREATE OR REPLACE FUNCTION update_node_notes_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_node_notes_updated_at
    BEFORE UPDATE ON node_notes
    FOR EACH ROW EXECUTE FUNCTION update_node_notes_updated_at();

CREATE INDEX node_notes_updated_at_idx ON node_notes(updated_at DESC);
