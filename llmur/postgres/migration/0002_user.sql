CREATE TYPE application_role AS ENUM ('admin', 'member');

CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,

    salt UUID NOT NULL,
    hashed_password TEXT NOT NULL,

    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    blocked BOOLEAN NOT NULL DEFAULT FALSE,

    role application_role NOT NULL DEFAULT 'member',

    created_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now())),
    updated_at TIMESTAMP NOT NULL DEFAULT (timezone('utc', now()))
);

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE
    ON users FOR EACH ROW
    EXECUTE PROCEDURE update_updated_at();

