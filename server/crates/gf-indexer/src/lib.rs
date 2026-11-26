//! Git scanning and indexing pipeline for GraphFinder Server.

pub mod git_clone;
pub mod git_source;

pub use git_clone::{clone_or_fetch_public_repo, GitCheckout, GitCloneError, GitCloneOptions};
pub use git_source::{
    validate_git_source_url, GitSourcePolicy, GitSourceValidationError, ValidatedGitUrl,
};
