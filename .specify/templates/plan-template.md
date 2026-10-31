# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]

**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 2021 (1.75+) + TypeScript 5.8+ (React 19)

**Primary Dependencies**: Tauri v2, fastembed 4, rusqlite 0.32, axum 0.7, rmcp 3.5.1, Tailwind CSS v4, Radix UI

**Storage**: Local plain Markdown notes in `./vault` / `./vaults` + SQLite (WAL mode) vector/graph database (`index.db`)

**Testing**: Cargo test (`cargo test`) for Rust engine/services + TypeScript verification (`tsc -b`, `npm run build`)

**Target Platform**: Cross-platform Desktop (Windows x64 NSIS/MSI, macOS Apple Silicon/Intel DMG, Linux deb/AppImage)

**Project Type**: Tauri v2 Desktop Application with Embedded MCP Server & Local Neural Embedding Engine

**Performance Goals**: Vector similarity retrieval < 50ms p95; 2D canvas graph interactive rendering at 60 FPS; instantaneous local-first state

**Constraints**: 100% offline-capable, local-first data sovereignty, zero cloud telemetry, atomic semantic chunking across headers (40-150 tokens)

**Scale/Scope**: Up to 10,000 notes / 50,000 code symbols per vault; multi-vault isolation support

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [ ] **I. Local-First & Plain Markdown**: Are files readable as plain Markdown with standard wikilinks?
- [ ] **II. Atomic Chunking & Token Economy**: Does the design avoid injecting full files and utilize deep links `[[Note#Section]]`?
- [ ] **III. Strict TDD**: Are automated test tasks identified before implementation code?
- [ ] **IV. Strict & Idiomatic Typing**: Are Rust errors explicitly handled (`Result`/`Option`) and TypeScript types strict (no `any`)?
- [ ] **V. MCP Native**: Are relevant features exposed or compatible with the Model Context Protocol server?

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
