CREATE TABLE rate_limit_buckets (
    bucket_hash text PRIMARY KEY,
    window_start timestamptz NOT NULL,
    request_count integer NOT NULL CHECK (request_count >= 0),
    expires_at timestamptz NOT NULL
);

CREATE INDEX idx_rate_limit_buckets_expiry ON rate_limit_buckets(expires_at);
