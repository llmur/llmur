CREATE TABLE IF NOT EXISTS virtual_keys (
    id UUID PRIMARY KEY,
    alias TEXT NOT NULL,
    description TEXT,
    salt UUID NOT NULL,
    encrypted_key TEXT NOT NULL,
    blocked BOOLEAN NOT NULL DEFAULT FALSE,

    project_id UUID NOT NULL,

    -- Limits
    budget_limits JSONB,
    request_limits JSONB,
    token_limits JSONB,

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),
    updated_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),

    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);
