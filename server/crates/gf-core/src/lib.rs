//! Core parsing, chunking, indexing and graph primitives for GraphFinder Server.
//!
//! This crate intentionally has no dependency on Tauri or any desktop runtime.

pub mod markdown {
    pub fn not_implemented() -> &'static str {
        "markdown parsing is not implemented yet"
    }
}
