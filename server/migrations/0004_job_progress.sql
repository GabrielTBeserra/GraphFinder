ALTER TABLE index_jobs
    ADD COLUMN progress_percent integer NOT NULL DEFAULT 0,
    ADD COLUMN current_stage text,
    ADD COLUMN heartbeat_at timestamptz,
    ADD COLUMN error_details jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD CONSTRAINT idx_index_jobs_progress_range CHECK (progress_percent BETWEEN 0 AND 100);
