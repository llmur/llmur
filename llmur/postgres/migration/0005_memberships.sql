CREATE TYPE project_role AS ENUM ('admin', 'developer', 'guest');

CREATE TABLE IF NOT EXISTS memberships (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    project_id UUID NOT NULL,
    role project_role NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),
    updated_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),
    UNIQUE (user_id, project_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);


CREATE TRIGGER memberships_updated_at
    BEFORE UPDATE
    ON memberships FOR EACH ROW
    EXECUTE PROCEDURE update_updated_at();
