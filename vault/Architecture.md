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
- Exposes tools like `search_semantic`, `get_compressed_context`, and `store_context` via stdio for AI coding assistants.
- See integration workflows in the [[Usage Guide]].

Tags: #architecture #rust #sqlite #embeddings #mcp
