-- Add users
CREATE TABLE IF NOT EXISTS users
(
    id            BIGSERIAL PRIMARY KEY,
    username      VARCHAR                  NOT NULL UNIQUE,
    email         VARCHAR                  NOT NULL UNIQUE,
    password_hash VARCHAR                  NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);