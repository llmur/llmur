CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),
    updated_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now()))
);

CREATE TRIGGER update_projects_updated_at
    BEFORE UPDATE
    ON projects FOR EACH ROW
    EXECUTE PROCEDURE update_updated_at();