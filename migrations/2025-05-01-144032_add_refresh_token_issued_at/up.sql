-- Your SQL goes here
ALTER TABLE "refresh_tokens"
    ADD issued_at TIMESTAMP NOT NULL DEFAULT NOW();
