use thiserror::Error;

use crate::TaskId;

#[derive(Debug, Clone, Error)]
pub enum PulsyncError {
    #[error("[{0}] Task ({1}): not found")]
    TaskNotFound(&'static str, TaskId),
    #[error("[pause] Task ({0}): already paused")]
    AlreadyPaused(TaskId),
    #[error("[resume] Task ({0}): already resumed")]
    AlreadyResumed(TaskId),
    #[error("[schedule] Task ({0}): already scheduled")]
    AlreadyScheduled(TaskId),
}
