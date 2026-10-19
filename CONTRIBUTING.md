# Contributing to GraphFinder

Thank you for your interest in contributing to GraphFinder. GraphFinder is a 100% open-source, local-first knowledge graph application engineered for data sovereignty and surgical token economy for AI agents.

This guide outlines the process and standards for submitting bug fixes, new features, and documentation improvements.

---

## Code of Conduct

By participating in this project, you agree to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md). We treat everyone with respect, empathy, and professional integrity.

---

## Ways to Contribute

1. **Report Bugs**: Open an issue detailing the unexpected behavior, minimal steps to reproduce, and your environment specifications (operating system, Node.js version, and Rust version).
2. **Suggest Enhancements**: Submit an issue describing your proposal and how it aligns with our principles of minimalism, low token overhead, and local-first architecture.
3. **Submit Code**: Pick an open issue or discuss an improvement with maintainers, then submit a pull request.
4. **Improve Documentation**: Enhance documentation, guides, setup steps, or MCP integration examples.

---

## Development Workflow

### 1. Prerequisites
* Node.js 18 or higher (LTS recommended)
* Rust and Cargo 1.75 or higher
* Git

### 2. Fork and Clone
```bash
git clone https://github.com/YOUR_USERNAME/GraphFinder.git
cd GraphFinder
git remote add upstream https://github.com/ORIGINAL_OWNER/GraphFinder.git
```

### 3. Install Dependencies
```bash
npm install
```

### 4. Create a Branch
Create a descriptive branch based on `main`:
* Bug fixes: `fix/short-description`
* New features: `feat/short-description`
* Documentation: `docs/short-description`

Example:
```bash
git checkout -b feat/mcp-filter-by-tag
```

### 5. Running the Local Environment
* Run the desktop application with Tauri and hot reloading:
  ```bash
  npm run app
  ```
* Run only the web frontend preview (Vite):
  ```bash
  npm run dev
  ```
* Run the embedded MCP server directly:
  ```bash
  npm run mcp
  ```

---

## Code Standards

### Frontend (React 19, TypeScript, Tailwind CSS v4)
* **Zero Emojis**: The project adheres to a strict, minimalist, technical, and understated aesthetic. Do not use emojis in UI text, code comments, or documentation.
* **Component Architecture**: Use functional React components with hooks, strict TypeScript typing, and accessible Radix UI primitives.
* **Design System**: Maintain consistency with the curated dark palette, subtle borders, and monospace accents. Avoid ad-hoc utilities or neon glows.
* **Type Checking and Linting**:
  ```bash
  npm run build
  npm run lint
  ```

### Backend (Rust, Tauri v2, SQLite, FastEmbed)
* **Idiomatic and Safe Rust**: Format code using `cargo fmt` and resolve all warnings from `cargo clippy`.
* **Concurrency and I/O**: Utilize Tokio for asynchronous tasks and ensure SQLite operations use WAL mode for concurrent read-write workloads.
* **Atomic Chunking**: Any changes to the vector indexing pipeline must preserve the strict semantic splitting of notes by `##` and `###` headers.

---

## Commit Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

* `feat`: A new feature or MCP tool
* `fix`: A bug fix
* `docs`: Documentation updates
* `style`: Code style or formatting changes (no logic changes)
* `refactor`: Internal refactoring with unchanged behavior
* `test`: Adding or correcting tests
* `chore`: Build system, tooling, or dependency updates

Example:
```bash
git commit -m "feat(mcp): add tag-based note filter to semantic search"
```

---

## Submitting a Pull Request

1. Rebase your branch against the latest upstream:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```
2. Verify all checks pass locally:
   ```bash
   npm run build
   npm run lint
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
3. Push your branch to your fork:
   ```bash
   git push origin feat/your-feature
   ```
4. Open a Pull Request on GitHub using the provided PR template.

---

## Licensing

By contributing to GraphFinder, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
