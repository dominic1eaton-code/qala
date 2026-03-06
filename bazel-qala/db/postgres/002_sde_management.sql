CREATE TABLE IF NOT EXISTS sdes (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    template_id UUID NOT NULL,
    status TEXT NOT NULL,
    snapshot_version INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sde_snapshots (
    id UUID PRIMARY KEY,
    sde_id UUID NOT NULL REFERENCES sdes(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    snapshot_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
