use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncPhase {
    Scanning,
    Indexing,
    Finalizing,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SyncProgress {
    pub phase: SyncPhase,
    pub completed: usize,
    pub total: usize,
    pub percent: Option<u8>,
    pub eta_seconds: Option<u64>,
    pub elapsed_ms: u64,
    pub current_file: Option<String>,
}

impl SyncProgress {
    pub fn scanning(completed: usize, elapsed_ms: u64, current_file: Option<String>) -> Self {
        Self {
            phase: SyncPhase::Scanning,
            completed,
            total: 0,
            percent: None,
            eta_seconds: None,
            elapsed_ms,
            current_file,
        }
    }

    pub fn indexing(
        completed: usize,
        total: usize,
        elapsed_ms: u64,
        current_file: Option<String>,
    ) -> Self {
        let percent = if total == 0 {
            0
        } else {
            ((completed as u128 * 100 / total as u128).min(99)) as u8
        };
        let eta_seconds = (completed > 0 && completed < total).then(|| {
            (elapsed_ms as u128 * (total - completed) as u128 / completed as u128 / 1_000)
                .min(u64::MAX as u128) as u64
        });

        Self {
            phase: SyncPhase::Indexing,
            completed,
            total,
            percent: Some(percent),
            eta_seconds,
            elapsed_ms,
            current_file,
        }
    }

    pub fn finalizing(completed: usize, total: usize, elapsed_ms: u64) -> Self {
        Self {
            phase: SyncPhase::Finalizing,
            completed,
            total,
            percent: Some(99),
            eta_seconds: None,
            elapsed_ms,
            current_file: None,
        }
    }

    pub fn complete(completed: usize, total: usize, elapsed_ms: u64) -> Self {
        Self {
            phase: SyncPhase::Complete,
            completed,
            total,
            percent: Some(100),
            eta_seconds: Some(0),
            elapsed_ms,
            current_file: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SyncPhase, SyncProgress};

    #[test]
    fn indexing_progress_calculates_percentage_and_eta() {
        let progress = SyncProgress::indexing(2, 10, 4_000, Some("src/lib.rs".to_string()));

        assert_eq!(progress.phase, SyncPhase::Indexing);
        assert_eq!(progress.percent, Some(20));
        assert_eq!(progress.eta_seconds, Some(16));
        assert_eq!(progress.current_file.as_deref(), Some("src/lib.rs"));
    }

    #[test]
    fn progress_without_completed_work_has_no_eta() {
        let progress = SyncProgress::indexing(0, 10, 4_000, None);

        assert_eq!(progress.percent, Some(0));
        assert_eq!(progress.eta_seconds, None);
    }

    #[test]
    fn scanning_progress_has_no_percentage_or_eta() {
        let progress = SyncProgress::scanning(12, 2_000, Some("src/lib.rs".to_string()));

        assert_eq!(progress.phase, SyncPhase::Scanning);
        assert_eq!(progress.completed, 12);
        assert_eq!(progress.total, 0);
        assert_eq!(progress.percent, None);
        assert_eq!(progress.eta_seconds, None);
    }

    #[test]
    fn completed_progress_reports_one_hundred_percent_and_zero_eta() {
        let progress = SyncProgress::complete(10, 10, 8_000);

        assert_eq!(progress.phase, SyncPhase::Complete);
        assert_eq!(progress.percent, Some(100));
        assert_eq!(progress.eta_seconds, Some(0));
    }
}
