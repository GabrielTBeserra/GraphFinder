ALTER TABLE chunks
    ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (to_tsvector('simple', content)) STORED;

CREATE INDEX idx_chunks_search_vector ON chunks USING gin(search_vector);
