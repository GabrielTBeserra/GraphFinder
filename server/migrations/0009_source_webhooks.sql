CREATE TABLE source_webhooks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id uuid NOT NULL UNIQUE REFERENCES sources(id) ON DELETE CASCADE,
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    provider text NOT NULL CHECK (provider IN ('github','gitlab','generic')),
    secret text NOT NULL,
    enabled boolean NOT NULL DEFAULT true,
    last_delivery_at timestamptz,
    last_status text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_source_webhooks_project ON source_webhooks(project_id);

WITH ranked_active_jobs AS (
    SELECT id,
           row_number() OVER (
               PARTITION BY source_id
               ORDER BY priority DESC, run_at, created_at DESC
           ) AS active_rank
    FROM index_jobs
    WHERE source_id IS NOT NULL
      AND status IN ('queued','running')
)
UPDATE index_jobs j
SET status='cancelled',
    finished_at=coalesce(j.finished_at,now()),
    updated_at=now(),
    error=coalesce(j.error,'superseded by duplicate active source job')
FROM ranked_active_jobs ranked
WHERE j.id=ranked.id
  AND ranked.active_rank > 1;

CREATE UNIQUE INDEX IF NOT EXISTS idx_index_jobs_one_active_per_source
    ON index_jobs(source_id)
    WHERE status IN ('queued','running');
