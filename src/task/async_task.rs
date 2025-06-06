use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

use chrono::NaiveDateTime;

use crate::{
    recurrence::Recurrence,
    task::{Task, TaskId},
};

use super::TaskStatus;

/// A trait for handling asynchronous tasks.
///
/// `run`:  Core method with the task actions
/// `save`: Method called after each run to let you save in database a copy of your state
pub trait AsyncTaskHandler: Task + Send {
    fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>>;
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
pub struct AsyncTask {
    pub(crate) id: TaskId,
    pub(crate) created_at: NaiveDateTime,
    pub(crate) status: Arc<RwLock<TaskStatus>>,
    pub(crate) title: Arc<String>,
    pub(crate) handle: tokio::task::JoinHandle<()>,
    pub(crate) recurrence: Arc<RwLock<Recurrence>>,
}

impl std::fmt::Display for AsyncTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[AsyncTask][{}] {} [{}]",
            self.id,
            self.title,
            *self.status.read().unwrap()
        )
    }
}
