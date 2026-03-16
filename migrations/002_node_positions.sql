CREATE TABLE IF NOT EXISTS node_positions (
    node_id    UUID             NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    x          DOUBLE PRECISION NOT NULL,
    y          DOUBLE PRECISION NOT NULL,
    updated_at TIMESTAMPTZ      NOT NULL DEFAULT now(),
    PRIMARY KEY (node_id)
);
