-- This file should undo anything in `up.sql`
ALTER TABLE "refresh_tokens"
    DROP COLUMN issued_at;