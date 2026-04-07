-- Allow tasks to exist without a parent node (standalone / inbox tasks).
-- Node association can be added or changed later via PATCH /tasks/{id}.
ALTER TABLE node_tasks ALTER COLUMN node_id DROP NOT NULL;
