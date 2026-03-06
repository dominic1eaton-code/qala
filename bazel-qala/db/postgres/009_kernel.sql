CREATE TABLE IF NOT EXISTS kernel_service_registry (
    service_name TEXT PRIMARY KEY,
    system_name TEXT NOT NULL,
    subsystem_name TEXT NOT NULL,
    status TEXT NOT NULL,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS kernel_event_counters (
    topic TEXT NOT NULL,
    source_service TEXT NOT NULL,
    event_count BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (topic, source_service)
);
