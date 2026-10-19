pub mod db;
pub mod embeddings;
pub mod parser;
pub mod project;
pub mod vault;

pub use db::{
    ChunkHit, CompressedContext, GraphData, GraphLink, GraphNode, LinkSuggestion, Neighborhood,
    NoteRecord, NoteSummary, SearchHit, SectionSummary,
};
pub use embeddings::EmbeddingEngine;
pub use project::{ProjectInfo, VaultManager};
pub use vault::Engine;
