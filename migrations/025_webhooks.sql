-- Webhook subscriptions: fire HTTP POST to a URL when node events occur.
CREATE TABLE IF NOT EXISTS webhooks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id    TEXT        NOT NULL,          -- Cognito sub of creator
    url         TEXT        NOT NULL,          -- Target URL to POST
    secret      TEXT,                          -- Optional shared secret for HMAC signing
    events      TEXT[]      NOT NULL DEFAULT ARRAY['node.created','node.updated','node.deleted'],
    is_active   BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_webhooks_owner_id ON webhooks(owner_id);
CREATE INDEX idx_webhooks_active ON webhooks(is_active) WHERE is_active = TRUE;
