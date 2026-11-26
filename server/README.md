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
