-- Your SQL goes here
CREATE TABLE IF NOT EXISTS "refresh_tokens" (
    token_str VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    expired_at TIMESTAMP NOT NULL,
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE
);
