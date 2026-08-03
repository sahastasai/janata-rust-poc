-- Local-only authentication proof for the isolated Rust migration.
-- No accounts, invite codes, credentials, or production data are seeded here.
PRAGMA foreign_keys = ON;

CREATE TABLE auth_users (
    id TEXT PRIMARY KEY NOT NULL,
    email_normalized TEXT NOT NULL,
    display_name TEXT NOT NULL,
    verification_level INTEGER NOT NULL DEFAULT 30,
    role TEXT NOT NULL DEFAULT 'member'
        CHECK (role IN ('member', 'sevak', 'admin')),
    token_version INTEGER NOT NULL DEFAULT 1 CHECK (token_version >= 1),
    password_salt TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    password_iterations INTEGER NOT NULL CHECK (password_iterations >= 100000),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    failed_login_count INTEGER NOT NULL DEFAULT 0 CHECK (failed_login_count >= 0),
    login_blocked_until INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX idx_auth_users_email_normalized
    ON auth_users(email_normalized);
CREATE INDEX idx_auth_users_active_email
    ON auth_users(is_active, email_normalized);
CREATE INDEX idx_auth_users_login_cooldown
    ON auth_users(login_blocked_until)
    WHERE login_blocked_until IS NOT NULL;

CREATE TABLE web_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    csrf_hash TEXT NOT NULL,
    token_version INTEGER NOT NULL CHECK (token_version >= 1),
    created_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    revoked_at INTEGER,
    FOREIGN KEY (user_id) REFERENCES auth_users(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_web_sessions_token_hash
    ON web_sessions(token_hash);
CREATE INDEX idx_web_sessions_user_active
    ON web_sessions(user_id, revoked_at, expires_at);
CREATE INDEX idx_web_sessions_expiry
    ON web_sessions(expires_at);
CREATE INDEX idx_web_sessions_revoked
    ON web_sessions(revoked_at)
    WHERE revoked_at IS NOT NULL;

CREATE TABLE invite_codes (
    code_hash TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    inviter_name TEXT,
    created_by_user_id TEXT,
    verification_level INTEGER NOT NULL DEFAULT 45,
    role TEXT NOT NULL DEFAULT 'member'
        CHECK (role IN ('member', 'sevak', 'admin')),
    max_uses INTEGER CHECK (max_uses IS NULL OR max_uses > 0),
    use_count INTEGER NOT NULL DEFAULT 0 CHECK (use_count >= 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    revoked_at INTEGER,
    FOREIGN KEY (created_by_user_id) REFERENCES auth_users(id) ON DELETE SET NULL
);

CREATE INDEX idx_invite_codes_active_expiry
    ON invite_codes(is_active, expires_at, revoked_at);
CREATE INDEX idx_invite_codes_creator
    ON invite_codes(created_by_user_id, created_at);
CREATE INDEX idx_invite_codes_expiry
    ON invite_codes(expires_at);
