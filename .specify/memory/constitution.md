# GraphFinder Constitution

<!--
  GraphFinder: Neural Knowledge Graph, Local Semantic Vault & Native MCP Server
  This constitution establishes mandatory architecture, testing, and quality rules
  for all Spec Kit workflows (/speckit-specify, /speckit-plan, /speckit-tasks, /speckit-implement).
-->

## Core Principles

### I. Local-First Data Sovereignty & Plain Markdown
All vault knowledge must reside in human-readable `.md` files in `./vault` (or isolated project workspaces in `./vaults/<project_id>`). Bidirectional connections use standard wikilinks `[[Note Title]]` or deep links `[[Note Title#Section]]`. Zero cloud dependencies or telemetry: all indexing, embedding, and vector retrieval executes 100% on-device.

### II. Atomic Semantic Chunking & Token Economy
Knowledge is indexed at atomic granularity strictly segmented across `##` and `###` Markdown headers (40 to 150 tokens per chunk). Every header generates an independent vector embedding in SQLite. AI agent tool calls must never ingest raw full files when surgical chunks via `[[Note#Section]]` suffice.

### III. Strict TDD (Test-Driven Development) & Non-Negotiable Quality Gates
Quality is enforced by automated test-first workflows:
1. **Red**: Write failing automated tests (Rust unit/integration tests with `cargo test` and/or frontend tests).
2. **Green**: Implement the minimal, clean code required to satisfy the tests.
3. **Refactor**: Clean up and optimize while preserving passing test suites.
Every task must pass `cargo test`, `cargo check`, TypeScript type-checking (`tsc -b`), and production build verification (`npm run build`).

### IV. Strict & Idiomatic Typing (TS + Rust)
- **Rust Backend**: Idiomatic Rust (2021 edition). Explicit error handling via `Result<T, E>` and `Option<T>` with zero unhandled panics or unwraps in production paths. Thread safety with Tokio async runtime and Arc/Mutex synchronization.
- **Frontend**: Strict TypeScript without `any` escapes. Strongly typed Tauri IPC invocation contracts with shared DTO schemas matching backend Rust serde models.

### V. Native MCP (Model Context Protocol) Integration
GraphFinder is a first-class MCP server. All knowledge graph querying, semantic search, neighborhood traversals, and context storage operations must remain accessible via native MCP tools (`rmcp` over streamable HTTP / stdio) with atomic context compression.

### VI. Topological Code & Knowledge Analytics
Maintain unified dual indexing for both Markdown notes and multi-language software source code (AST-Lite). Graph algorithms (Label Propagation communities, God Node identification, BFS pathfinding) run in pure Rust for real-time responsiveness.

## Technology Stack & Constraints

- **Desktop Framework**: Tauri v2 (`@tauri-apps/api` 2.12.4, Tauri CLI 2.12.3).
- **Backend Runtime**: Rust 2021, Tokio, Axum (MCP HTTP), rmcp 3.5.1 (official Model Context Protocol Rust SDK).
- **Embedded Database**: SQLite (via `rusqlite` bundled) in WAL mode (`PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;`).
- **Embedding Engine**: FastEmbed (ONNX Runtime) generating local dense embeddings (`all-MiniLM-L6-v2` or multilingual).
- **Frontend Stack**: React 19, TypeScript 5.8+, Vite 8, Tailwind CSS v4, Phosphor Icons, Radix UI primitives.
- **Target Platforms**: Windows (NSIS `.exe`, `.msi`), macOS (Apple Silicon `.dmg`), Linux (`.deb`, `.AppImage`).
- **Performance Budget**: Semantic vector search < 50ms p95; 2D canvas graph rendering at 60 FPS up to 10,000 nodes/edges.

## Development Workflow & Quality Gates

1. **Feature Specification (`/speckit-specify`)**:
   - Define user stories prioritized (P1, P2, P3).
   - Each story must have independent testable acceptance criteria (Given/When/Then).
2. **Implementation Planning (`/speckit-plan`)**:
   - Check compliance against this Constitution.
   - Specify Rust structs, SQLite migrations, and TypeScript interface contracts before coding.
3. **Task Generation (`/speckit-tasks`)**:
   - TDD tasks are mandatory: test scaffolding precedes implementation tasks.
   - Tasks must be dependency-ordered and grouped by User Story.
4. **Execution & Implementation (`/speckit-implement`)**:
   - Execute Red-Green-Refactor loop.
   - Run verification suite: `npm run lint`, `tsc -b`, `npm run build`, and `cargo check`/`cargo test`.

## Governance

This Constitution supersedes all ad-hoc conventions and unwritten rules.
- Any architectural amendment requires updating this document (`.specify/memory/constitution.md`) and synchronizing with `AGENTS.md`.
- Complexity must be justified; YAGNI and token economy principles apply to all features.

**Version**: 1.0.0 | **Ratified**: 2026-10-27 | **Last Amended**: 2026-10-27
