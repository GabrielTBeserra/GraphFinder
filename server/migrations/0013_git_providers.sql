CREATE TABLE git_providers (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name text NOT NULL,
    provider text NOT NULL CHECK (provider IN ('github','gitlab','azure_devops','generic')),
    host text NOT NULL,
    username text NOT NULL,
    secret_ciphertext bytea NOT NULL,
    secret_nonce bytea NOT NULL,
    secret_hint text NOT NULL,
    last_used_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)
);

CREATE INDEX idx_git_providers_tenant ON git_providers(tenant_id);
CREATE INDEX idx_git_providers_host ON git_providers(tenant_id, host);
