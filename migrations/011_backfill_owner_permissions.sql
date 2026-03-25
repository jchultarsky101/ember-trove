-- Migration 011: Backfill owner permissions
--
-- Every node that exists before the multi-user permission system was activated
-- needs an Owner permission row for its original creator (owner_id).  Without
-- this, the node's creator would be locked out as soon as require_role() starts
-- enforcing the permissions table.
--
-- The INSERT uses ON CONFLICT DO NOTHING so it is idempotent and safe to re-run.

INSERT INTO permissions (node_id, subject_id, role, granted_by)
SELECT id, owner_id, 'owner'::permission_role, owner_id
FROM nodes
ON CONFLICT (node_id, subject_id) DO NOTHING;
