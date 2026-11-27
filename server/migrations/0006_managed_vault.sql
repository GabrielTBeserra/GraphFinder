ALTER TABLE documents
    ADD COLUMN IF NOT EXISTS revision bigint NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now();

INSERT INTO index_generations (project_id, generation, status, published_at)
SELECT p.id, 0, 'published', now()
FROM projects p
ON CONFLICT (project_id, generation) DO NOTHING;
