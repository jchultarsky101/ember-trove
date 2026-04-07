-- Recurring tasks: 'daily' | 'weekly' | 'biweekly' | 'monthly' | 'yearly'
ALTER TABLE node_tasks ADD COLUMN recurrence TEXT;

-- Manual sort order for drag-to-reorder in My Day (0 = default/unset)
ALTER TABLE node_tasks ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
