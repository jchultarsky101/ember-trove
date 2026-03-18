-- ── Task management ─────────────────────────────────────────────────────────

CREATE TYPE task_status AS ENUM ('open', 'in_progress', 'done', 'cancelled');
CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high');

CREATE TABLE node_tasks (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id     UUID        NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    owner_id    UUID        NOT NULL,
    title       TEXT        NOT NULL CHECK (char_length(title) BETWEEN 1 AND 500),
    status      task_status NOT NULL DEFAULT 'open',
    priority    task_priority NOT NULL DEFAULT 'medium',
    -- focus_date: when non-null, this task appears in the user's "My Day" for that date
    focus_date  DATE,
    due_date    DATE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX node_tasks_node_id_idx   ON node_tasks(node_id);
CREATE INDEX node_tasks_owner_id_idx  ON node_tasks(owner_id);
CREATE INDEX node_tasks_focus_date_idx ON node_tasks(owner_id, focus_date)
    WHERE focus_date IS NOT NULL;
CREATE INDEX node_tasks_due_date_idx  ON node_tasks(due_date)
    WHERE due_date IS NOT NULL;

-- Keep updated_at current automatically
CREATE OR REPLACE FUNCTION update_node_tasks_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_node_tasks_updated_at
    BEFORE UPDATE ON node_tasks
    FOR EACH ROW EXECUTE FUNCTION update_node_tasks_updated_at();
