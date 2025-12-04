CREATE TYPE deployment_access AS ENUM ('private', 'public');

CREATE TABLE IF NOT EXISTS deployments (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,

    access deployment_access NOT NULL DEFAULT 'private',

    -- Limits
    budget_limits JSONB,
    request_limits JSONB,
    token_limits JSONB,

    created_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),
    updated_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now()))
);


CREATE TRIGGER update_deployments_updated_at
    BEFORE UPDATE
    ON deployments FOR EACH ROW
    EXECUTE PROCEDURE update_updated_at();