CREATE TABLE IF NOT EXISTS project_invites (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    code TEXT NOT NULL,

    assign_role project_role NOT NULL,

    valid_until TIMESTAMP with TIME ZONE,
    created_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now()))
);
