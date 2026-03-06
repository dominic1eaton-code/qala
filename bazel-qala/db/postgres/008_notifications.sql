CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY,
    target_id UUID NOT NULL,
    type TEXT NOT NULL,
    channel TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
