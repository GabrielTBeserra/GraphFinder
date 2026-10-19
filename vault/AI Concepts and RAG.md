# AI Concepts and Atomic RAG

How GraphFinder transforms standard Markdown into a high-density, token-efficient knowledge base for LLMs:

## 1. Atomic Chunking by Headers
- Rather than vectorizing entire documents or arbitrary character windows, GraphFinder splits notes strictly across `##` and `###` headers.
- Each chunk averages 40 to 150 tokens, creating fine-grained vectors with clear semantic boundaries.

## 2. Deep Linking and Context Economy
- When an AI agent performs a semantic query, the MCP server returns only the exact matching subsection with a `[[Note#Section]]` deep link.
- This preserves context window budget and eliminates prompt degradation.
- Review technical implementation in [[Architecture]].

Tags: #ai #rag #embeddings #architecture
