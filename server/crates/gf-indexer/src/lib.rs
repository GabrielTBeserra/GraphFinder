//! Git scanning and indexing pipeline for GraphFinder Server.

pub mod git_source;

pub use git_source::{
    validate_git_source_url, GitSourcePolicy, GitSourceValidationError, ValidatedGitUrl,
};
