CREATE TYPE deployment_access AS ENUM ('private', 'public');
CREATE TYPE load_balancing_strategy AS ENUM ('round_robin', 'weighted_round_robin', 'least_connections', 'weighted_least_connections');

CREATE TABLE IF NOT EXISTS deployments (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,

    access deployment_access NOT NULL DEFAULT 'private',
    strategy load_balancing_strategy NOT NULL DEFAULT 'round_robin',

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