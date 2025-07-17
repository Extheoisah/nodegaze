CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), -- UUID-like ID
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    deleted_at DATETIME DEFAULT NULL,
    UNIQUE(name)
);

CREATE INDEX idx_accounts_name ON accounts(name);

CREATE TRIGGER accounts_updated_at
    AFTER UPDATE ON accounts
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE accounts SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), -- UUID-like ID
    account_id TEXT NOT NULL,
    name TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    role_id TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    deleted_at DATETIME DEFAULT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE SET NULL
);

CREATE INDEX idx_users_account_id ON users(account_id);
CREATE INDEX idx_users_name ON users(name);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role_id ON users(role_id);

CREATE TRIGGER users_updated_at
    AFTER UPDATE ON users
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), -- UUID-like ID
    name TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    deleted_at DATETIME DEFAULT NULL,
    UNIQUE(name)
);

CREATE INDEX idx_roles_name ON roles(name);

CREATE TRIGGER roles_updated_at
    AFTER UPDATE ON roles
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE roles SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

INSERT OR IGNORE INTO roles (name) VALUES ('Admin'), ('Member');

CREATE TABLE IF NOT EXISTS credentials (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), -- UUID-like ID
    user_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    node_alias TEXT DEFAULT '',
    macaroon TEXT NOT NULL,
    tls_cert TEXT NOT NULL,
    address TEXT NOT NULL,
    node_type TEXT DEFAULT 'lnd',
    client_cert TEXT DEFAULT NULL,
    client_key TEXT DEFAULT NULL,
    ca_cert TEXT DEFAULT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    deleted_at DATETIME DEFAULT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_credentials_user_id ON credentials(user_id);
CREATE INDEX idx_credentials_account_id ON credentials(account_id);
CREATE INDEX idx_credentials_node_type ON credentials(node_type);

CREATE UNIQUE INDEX idx_credentials_user_unique ON credentials(user_id) WHERE is_deleted = 0;

CREATE TRIGGER credentials_updated_at
    AFTER UPDATE ON credentials
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE credentials SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
