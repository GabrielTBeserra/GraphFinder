CREATE TABLE oidc_auth_states (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    provider text NOT NULL DEFAULT 'default',
    tenant_slug text NOT NULL,
    state_hash text NOT NULL UNIQUE,
    nonce text NOT NULL,
    pkce_verifier text NOT NULL,
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_oidc_auth_states_expiry ON oidc_auth_states(expires_at) WHERE consumed_at IS NULL;

CREATE TABLE oidc_identities (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    issuer text NOT NULL,
    subject text NOT NULL,
    email text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, issuer, subject)
);

CREATE INDEX idx_oidc_identities_user ON oidc_identities(user_id);
