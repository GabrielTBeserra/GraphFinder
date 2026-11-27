# GraphFinder Server

GraphFinder Server is the additive, multi-user server variant of GraphFinder.

The desktop application in `../src` and `../src-tauri` remains unchanged. This tree contains an independent Rust workspace and must not depend on Tauri, WebKit or a graphical runtime.

## Planned commands

```bash
cargo run --manifest-path server/Cargo.toml -- migrate
cargo run --manifest-path server/Cargo.toml -- api
cargo run --manifest-path server/Cargo.toml -- worker
```

## Implementation plan

See [`IMPLEMENTATION_CHECKLIST.md`](./IMPLEMENTATION_CHECKLIST.md).


## Private Git providers and indexing automation

The server can index private HTTPS repositories through centrally stored provider credentials for GitHub, GitLab, Azure DevOps, self-hosted/generic Git servers, or other HTTPS-compatible platforms.

Operational requirements:

```bash
# 32 bytes as hex/base64/raw text; example:
export GF_CREDENTIALS_ENCRYPTION_KEY=$(openssl rand -hex 32)

# optional, only for trusted private/self-hosted network hosts
export GF_ALLOW_PRIVATE_GIT_HOSTS=false
export GF_ALLOWED_GIT_HOSTS=git.example.internal
```

Secrets are encrypted at rest using AES-256-GCM. The API never returns raw provider tokens. Back up the encryption key with the deployment secrets: rotating or losing it makes existing provider credentials unreadable.

In the admin panel:

1. Open **Providers** and create a named credential using the exact repository host.
2. Open **Sources**, choose the provider when adding the repository.
3. Select manual indexing, a periodic interval, or **after push · webhook**.
4. For after-push mode, copy the generated endpoint and one-time webhook secret into the Git platform webhook/service-hook configuration.

Supported webhook validation modes:

- GitHub: `X-Hub-Signature-256` HMAC-SHA256;
- GitLab: `X-Gitlab-Token`;
- Azure DevOps: Basic-auth password on service hooks;
- generic: `X-Graphfinder-Signature` HMAC-SHA256.

The payload repository URL and branch are matched before jobs are enqueued. Interval scheduling supports 5 minutes to 24 hours and is processed by the worker.
