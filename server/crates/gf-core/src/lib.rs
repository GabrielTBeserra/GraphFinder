//! Core parsing, chunking, indexing and graph primitives for GraphFinder Server.
//!
//! This crate intentionally has no dependency on Tauri or any desktop runtime.

pub mod code;
pub mod markdown;

pub use code::{detect_language, parse_code, CodeSymbol, ParsedCode};
pub use markdown::{content_hash, estimate_tokens, parse_markdown, MarkdownChunk, MarkdownNote};
