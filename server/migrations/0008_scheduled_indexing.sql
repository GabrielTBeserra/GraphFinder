ALTER TABLE sources
    ADD COLUMN IF NOT EXISTS auto_index_interval_seconds integer,
    ADD COLUMN IF NOT EXISTS next_index_at timestamptz;

CREATE INDEX IF NOT EXISTS idx_sources_due_index
    ON sources(next_index_at)
    WHERE auto_index_interval_seconds IS NOT NULL;
