# System Architecture

GraphFinder is built upon three high-density architectural pillars designed for data sovereignty and surgical token efficiency:

## 1. Local Storage and SQLite
- **Plain Markdown (`.md`) Files**: Stored directly in your local vault directory. 100% human-readable with standard wikilinks [[Link]].
- **Local SQLite Engine**: Stores nodes, relational edges, and vector embeddings in WAL mode for concurrent, low-latency queries.

## 2. Rust Semantic Engine
- Powered by **FastEmbed** and ONNX Runtime for offline embeddings with zero cloud dependencies.
- Implements graph traversal algorithms (k-hop expansion) to extract focused neighborhoods around any query or note.
- Learn more in [[Welcome]].

## 3. Model Context Protocol (MCP) Server
- Exposes 20 surgical tools (`search_semantic`, `search_code`, `get_file_context`) for AI coding assistants.
- Operates concurrently in Streamable HTTP/SSE and Headless Stdio CLI modes.
- See integration workflows in the [[Usage Guide]].

## 4. AST-Lite Multi-Language Code Engine
- Indexes entire project repositories across 15+ programming languages (Rust, TypeScript, Python, Go, C/C++, etc.).
- Parses module import statements to construct a bidirectional code dependency graph.
- Generates atomic symbol embeddings for functions and structs, enabling sub-80-token structural inspection.

Tags: #architecture #rust #sqlite #embeddings #mcp #code #ast

