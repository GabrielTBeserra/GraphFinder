pub mod code;
pub mod db;
pub mod embeddings;
pub mod graph_analytics;
pub mod parser;
pub mod project;
pub mod progress;
pub mod vault;

pub use code::{
    CodeChunk, CodeRelationKind, ExtractedCodeRelation, ExtractedImport, ParsedCodeFile,
};
pub use db::{
    ChunkHit, CodeChunkHit, CodeDependencies, CodeFileRecord, CodeRelationInfo, CodeSearchHit,
    CompressedContext, FileContext, GraphData, GraphLink, GraphNode, LinkSuggestion, Neighborhood,
    NoteRecord, NoteSummary, SearchHit, SectionSummary, SymbolInfo, VaultStats,
};
pub use embeddings::EmbeddingEngine;
pub use graph_analytics::{
    CommunityInfo, DependencyCycle, ExportSummary, GodNodeInfo, GraphAnalysisReport,
    GraphAnalyticsEngine, NodeExplanation, PathResult, PathStep, SurpriseEdge,
};
pub use project::{clean_path_str, ProjectInfo, VaultManager};
pub use progress::{SyncPhase, SyncProgress};
pub use vault::Engine;
