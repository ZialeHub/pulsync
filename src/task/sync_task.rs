use std::sync::{Arc, RwLock};

use chrono::NaiveDateTime;

use crate::{
    recurrence::Recurrence,
    task::{Task, TaskId, TaskStatus},
};

/// A trait for handling asynchronous tasks.
pub trait SyncTaskHandler: Task + Send {
    fn run(&self);
}

/// Represents an asynchronous task, scheduled in the scheduler.
///
/// # Fields
/// * `id` - The unique identifier of the task.
/// * `created_at` - The date and time the task was created.
/// * `status` - The status of the task (to pause/resume the execution).
/// * `title` - The title of the task, used to identify the task in the scheduler list.
/// * `handler` - The handler of the task.
/// * `recurrence` - The recurrence of the task.
pub struct SyncTask {
    pub(crate) id: TaskId,
    pub(crate) created_at: NaiveDateTime,
    pub(crate) status: Arc<RwLock<TaskStatus>>,
    pub(crate) title: Arc<String>,
    pub(crate) _handler: std::thread::JoinHandle<()>,
    pub(crate) recurrence: Arc<RwLock<Recurrence>>,
}

impl std::fmt::Display for SyncTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[SyncTask({})] {} started at {}",
            self.id,
            self.title,
            self.created_at.date()
        )
    }
}
