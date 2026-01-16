-- Define the available providers
CREATE TYPE provider AS ENUM ('azure/openai', 'openai/v1', 'gemini');
CREATE TYPE azure_openai_api_version AS ENUM ('2024-10-21');
CREATE TYPE gemini_api_version AS ENUM ('v1beta');

-- External connections table
CREATE TABLE connections (
    id UUID PRIMARY KEY,
    connection_info JSONB NOT NULL,

    -- Limits
    budget_limits JSONB,
    request_limits JSONB,
    token_limits JSONB,

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),
    updated_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now()))
);


CREATE TRIGGER update_connections_updated_at
    BEFORE UPDATE
    ON connections FOR EACH ROW
    EXECUTE PROCEDURE update_updated_at();
