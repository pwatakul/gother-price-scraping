-- Migration 026: application users for login (REQ-009)
--
-- The table only. No seeded row here: an Argon2 hash carries its own salt,
-- so seeding from SQL would commit one specific password hash to git forever
-- and make ADMIN_PASSWORD unable to override it. The admin user is created at
-- startup instead (see main.rs), which is idempotent and env-overridable.

CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username        VARCHAR(64)  NOT NULL,
    password_hash   TEXT         NOT NULL,
    role            VARCHAR(20)  NOT NULL DEFAULT 'viewer',
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT users_role_check CHECK (role IN ('admin', 'viewer'))
);

-- Usernames are matched case-insensitively at login, so uniqueness has to be
-- case-insensitive too — otherwise 'Admin' and 'admin' could both exist and
-- the lookup would be ambiguous.
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username_lower ON users (LOWER(username));

COMMENT ON TABLE users IS 'Application login accounts. Roles are stored and displayed but not yet enforced (REQ-009 v1.0).';
COMMENT ON COLUMN users.password_hash IS 'Argon2id PHC string, salt included.';
