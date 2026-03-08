-- ALICE Cache Cloud: Domain-specific tables
CREATE TABLE IF NOT EXISTS cache_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    name TEXT NOT NULL,
    memory_mb INTEGER NOT NULL DEFAULT 256,
    max_memory_mb INTEGER NOT NULL DEFAULT 16384,
    eviction_policy TEXT NOT NULL DEFAULT 'tinylfu' CHECK (eviction_policy IN ('tinylfu', 'lru', 'lfu', 'random')),
    prefetch_enabled BOOLEAN NOT NULL DEFAULT true,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('provisioning', 'active', 'suspended', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS cache_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id UUID NOT NULL REFERENCES cache_instances(id) ON DELETE CASCADE,
    period_start TIMESTAMPTZ NOT NULL,
    hits BIGINT NOT NULL DEFAULT 0,
    misses BIGINT NOT NULL DEFAULT 0,
    prefetch_hits BIGINT NOT NULL DEFAULT 0,
    evictions BIGINT NOT NULL DEFAULT 0,
    bytes_stored BIGINT NOT NULL DEFAULT 0,
    hit_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0
);

CREATE INDEX idx_cache_instances_user ON cache_instances(user_id);
CREATE INDEX idx_cache_metrics_instance ON cache_metrics(instance_id, period_start);
