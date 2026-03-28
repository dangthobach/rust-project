-- PostgreSQL 17: clients
CREATE TABLE IF NOT EXISTS clients (
    id UUID PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    company TEXT,
    position TEXT,
    address TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    notes TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_clients_status ON clients(status);
CREATE INDEX IF NOT EXISTS idx_clients_assigned_to ON clients(assigned_to);
CREATE INDEX IF NOT EXISTS idx_clients_email ON clients(email);
CREATE INDEX IF NOT EXISTS idx_clients_name ON clients(name);
CREATE INDEX IF NOT EXISTS idx_clients_created_at ON clients(created_at DESC);

DROP TRIGGER IF EXISTS update_clients_updated_at ON clients;
CREATE TRIGGER update_clients_updated_at
BEFORE UPDATE ON clients
FOR EACH ROW
EXECUTE FUNCTION trg_set_updated_at();
