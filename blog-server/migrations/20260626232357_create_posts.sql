-- Add posts
CREATE TABLE IF NOT EXISTS posts
(
    id         BIGSERIAL PRIMARY KEY,
    title      VARCHAR                  NOT NULL,
    content    TEXT                     NOT NULL,
    author_id  BIGINT                   NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_users_author_id ON posts (author_id);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON posts (created_at);

ALTER TABLE posts
    ADD CONSTRAINT posts_author_id_fk FOREIGN KEY (author_id) REFERENCES users (id)
        ON DELETE CASCADE;
