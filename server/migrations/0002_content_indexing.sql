CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE sources (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    kind text NOT NULL,
    uri text NOT NULL,
    branch text,
    credential_ref text,
    include_patterns text[] NOT NULL DEFAULT '{}',
    exclude_patterns text[] NOT NULL DEFAULT '{}',
    config jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (id, project_id)
);

CREATE UNIQUE INDEX idx_sources_project_uri_branch
    ON sources(project_id, uri, COALESCE(branch, ''));

CREATE INDEX idx_sources_project ON sources(project_id);

CREATE TABLE index_generations (
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    generation bigint NOT NULL,
    status text NOT NULL DEFAULT 'pending',
    embedding_model text NOT NULL DEFAULT 'multilingual-e5-small',
    embedding_dimensions integer NOT NULL DEFAULT 384,
    source_revisions jsonb NOT NULL DEFAULT '{}'::jsonb,
    error text,
    created_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    published_at timestamptz,
    PRIMARY KEY (project_id, generation),
    CHECK (status IN ('pending', 'running', 'publishing', 'published', 'failed', 'archived'))
);

CREATE INDEX idx_index_generations_status
    ON index_generations(project_id, status, generation DESC);

CREATE TABLE documents (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL,
    generation bigint NOT NULL,
    source_id uuid,
    path text NOT NULL,
    document_type text NOT NULL,
    language text,
    title text,
    content_hash text NOT NULL,
    size_bytes bigint NOT NULL DEFAULT 0,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (project_id, generation)
        REFERENCES index_generations(project_id, generation) ON DELETE CASCADE,
    FOREIGN KEY (source_id, project_id)
        REFERENCES sources(id, project_id) ON DELETE SET NULL (source_id),
    UNIQUE (project_id, generation, id),
    UNIQUE (project_id, generation, path)
);

CREATE INDEX idx_documents_project_generation
    ON documents(project_id, generation);
CREATE INDEX idx_documents_source ON documents(source_id);
CREATE INDEX idx_documents_content_hash ON documents(project_id, generation, content_hash);
CREATE INDEX idx_documents_type_language ON documents(project_id, generation, document_type, language);

CREATE TABLE chunks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id uuid NOT NULL,
    project_id uuid NOT NULL,
    generation bigint NOT NULL,
    chunk_index integer NOT NULL,
    heading_path text[] NOT NULL DEFAULT '{}',
    content text NOT NULL,
    token_count integer NOT NULL DEFAULT 0,
    content_hash text NOT NULL,
    start_line integer,
    end_line integer,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (project_id, generation, document_id)
        REFERENCES documents(project_id, generation, id) ON DELETE CASCADE,
    UNIQUE (project_id, generation, id),
    UNIQUE (document_id, chunk_index)
);

CREATE INDEX idx_chunks_project_generation ON chunks(project_id, generation);
CREATE INDEX idx_chunks_document ON chunks(document_id);
CREATE INDEX idx_chunks_content_hash ON chunks(project_id, generation, content_hash);

CREATE TABLE chunk_embeddings (
    chunk_id uuid PRIMARY KEY,
    project_id uuid NOT NULL,
    generation bigint NOT NULL,
    model text NOT NULL,
    embedding vector(384) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (project_id, generation, chunk_id)
        REFERENCES chunks(project_id, generation, id) ON DELETE CASCADE
);

CREATE INDEX idx_chunk_embeddings_project_generation
    ON chunk_embeddings(project_id, generation);
CREATE INDEX idx_chunk_embeddings_hnsw
    ON chunk_embeddings USING hnsw (embedding vector_cosine_ops);

CREATE TABLE links (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL,
    generation bigint NOT NULL,
    source_document_id uuid NOT NULL,
    target_document_id uuid,
    target_path text NOT NULL,
    kind text NOT NULL,
    label text,
    line_number integer,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (project_id, generation, source_document_id)
        REFERENCES documents(project_id, generation, id) ON DELETE CASCADE,
    FOREIGN KEY (project_id, generation, target_document_id)
        REFERENCES documents(project_id, generation, id) ON DELETE SET NULL (target_document_id)
);

CREATE INDEX idx_links_project_generation ON links(project_id, generation);
CREATE INDEX idx_links_source ON links(source_document_id);
CREATE INDEX idx_links_target ON links(target_document_id);
CREATE INDEX idx_links_target_path ON links(project_id, generation, target_path);

CREATE TABLE index_jobs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    source_id uuid REFERENCES sources(id) ON DELETE SET NULL,
    generation bigint,
    status text NOT NULL DEFAULT 'queued',
    priority integer NOT NULL DEFAULT 0,
    attempts integer NOT NULL DEFAULT 0,
    max_attempts integer NOT NULL DEFAULT 3,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    error text,
    run_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    finished_at timestamptz,
    created_by uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    FOREIGN KEY (project_id, generation)
        REFERENCES index_generations(project_id, generation) ON DELETE SET NULL (generation)
);

CREATE INDEX idx_index_jobs_project ON index_jobs(project_id, created_at DESC);
CREATE INDEX idx_index_jobs_queue
    ON index_jobs(status, priority DESC, run_at, created_at)
    WHERE status = 'queued';
CREATE INDEX idx_index_jobs_generation ON index_jobs(project_id, generation);
