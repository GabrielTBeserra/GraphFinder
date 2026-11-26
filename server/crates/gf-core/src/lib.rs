//! Core parsing, chunking, indexing and graph primitives for GraphFinder Server.
//!
//! This crate intentionally has no dependency on Tauri or any desktop runtime.

pub mod markdown;

pub use markdown::{content_hash, estimate_tokens, parse_markdown, MarkdownChunk, MarkdownNote};
