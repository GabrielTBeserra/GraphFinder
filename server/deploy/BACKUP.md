# GraphFinder Server backup and restore

## Backup scope

The PostgreSQL database is the authoritative backup object. It contains tenants, users, token hashes, projects, sources, indexed documents/chunks, embeddings, links, generations, and indexing jobs.

Do not treat these directories as authoritative backup data:

- `model-cache`: downloadable FastEmbed/ONNX model artifacts;
- `repo-cache`: reconstructible Git clones;
- Caddy data: TLS certificates and runtime state that can be reissued.

They may be excluded from backup and rebuilt after restore.

## Creating a backup

From `server/deploy`, with the compose environment loaded:

```bash
export POSTGRES_PASSWORD='...'
./backup.sh
```

This writes a compressed PostgreSQL custom-format dump to `backups/` by default. Store dumps outside the host and encrypt them at rest because token hashes and user data are sensitive even though plaintext tokens/passwords are not stored.

## Restoring

Restore is destructive to the target database. Create a fresh backup first when possible, then run:

```bash
export POSTGRES_PASSWORD='...'
GF_CONFIRM_RESTORE=yes ./restore.sh backups/graphfinder-YYYYMMDDTHHMMSSZ.dump
```

After restore:

1. run `docker compose up -d`;
2. verify `/ready` through the proxy;
3. sign in to the admin panel;
4. verify projects, sources, jobs, and search results;
5. reindex only if content validation indicates corruption.

## Retention guidance

A minimal production policy is daily backups retained for 14 days plus weekly backups retained for 8 weeks. Adjust retention to compliance and storage requirements. Test restores regularly in an isolated database or compose project.
