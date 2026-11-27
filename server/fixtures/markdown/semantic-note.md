---
title: Semantic Indexing
tags: [rag, architecture]
aliases: [semantic-index]
---

# Semantic Indexing

Semantic indexing splits durable knowledge into small retrievable units. This note links to [[Graph Model]] and uses #search as a document-level tag.

## 1. Chunking Rules

Chunks are split by semantic headings and should stay small enough for embedding models. See [[Atomic Notes#Headers]] for the writing contract.

Tags: #chunking

```rust
// [[Fake Link]] and #fake must never be indexed as real metadata.
fn example() {}
```

### 1.1 Deep Links

Deep links point to exact sections, such as [[Semantic Indexing#1. Chunking Rules]], reducing token usage for agents.

#### Nested detail

This H4 heading must remain inside the parent H3 chunk instead of creating a new boundary.

## 2. Operational Concerns

Indexes are derived artifacts. Markdown files remain the portable source of truth for humans, agents and version control.
