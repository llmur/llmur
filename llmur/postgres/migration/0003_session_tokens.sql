CREATE TABLE session_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    expires_at TIMESTAMP with TIME ZONE,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP with TIME ZONE NOT NULL DEFAULT (timezone('utc', now())),
    updated_at TIMESTAMP with TIME ZONE NOT NULL DEFAULT (timezone('utc', now())),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- CREATE TRIGGER update_projects_updated_at
--    BEFORE UPDATE
--    ON session_tokens FOR EACH ROW
--    EXECUTE PROCEDURE update_updated_at();

-- CREATE OR REPLACE FUNCTION limit_user_tokens()
-- RETURNS TRIGGER AS $$
-- BEGIN
--     DELETE FROM session_tokens
--     WHERE user_id = NEW.user_id
--     AND id NOT IN (
--         SELECT id FROM session_tokens
--         WHERE user_id = NEW.user_id
--         ORDER BY created_at DESC
--         LIMIT 5
--     );
--     RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;
--
-- CREATE TRIGGER limit_user_tokens_trigger
-- AFTER INSERT ON session_tokens
-- FOR EACH ROW EXECUTE FUNCTION limit_user_tokens();
