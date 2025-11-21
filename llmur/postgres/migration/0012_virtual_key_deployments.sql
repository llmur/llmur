CREATE TABLE IF NOT EXISTS virtual_keys_deployments_map (
    id UUID PRIMARY KEY,
    virtual_key_id UUID NOT NULL,
    deployment_id UUID NOT NULL,
    UNIQUE (virtual_key_id, deployment_id),
    FOREIGN KEY (virtual_key_id) REFERENCES virtual_keys (id) ON DELETE CASCADE,
    FOREIGN KEY (deployment_id) REFERENCES deployments (id) ON DELETE CASCADE
);
