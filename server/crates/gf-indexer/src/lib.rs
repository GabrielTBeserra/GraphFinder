//! Git scanning and indexing pipeline for GraphFinder Server.

pub mod git_clone;
pub mod git_source;
pub mod pipeline;
pub mod scanner;
pub mod worker;

pub use git_clone::{
    clone_or_fetch_public_repo, GitAuth, GitCheckout, GitCloneError, GitCloneOptions,
};
pub use git_source::{
    validate_git_source_url, GitSourcePolicy, GitSourceValidationError, ValidatedGitUrl,
};

pub use scanner::{diff_scan, scan_repository, ScanDiff, ScanError, ScannedFile};

pub use pipeline::{dry_run_checkout, index_checkout, IndexingError, IndexingReport};

pub use worker::{process_next_index_job, WorkerConfig, WorkerError};
