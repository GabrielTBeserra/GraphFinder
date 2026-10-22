pub mod code;
pub mod db;
pub mod embeddings;
pub mod parser;
pub mod project;
pub mod vault;

pub use code::{CodeChunk, ExtractedImport, ParsedCodeFile};
pub use db::{
    ChunkHit, CodeChunkHit, CodeDependencies, CodeFileRecord, CodeSearchHit, CompressedContext,
    FileContext, GraphData, GraphLink, GraphNode, LinkSuggestion, Neighborhood, NoteRecord,
    NoteSummary, SearchHit, SectionSummary, SymbolInfo, VaultStats,
};
pub use embeddings::EmbeddingEngine;
pub use project::{clean_path_str, ProjectInfo, VaultManager};
pub use vault::Engine;

