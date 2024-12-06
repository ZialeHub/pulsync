use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::unique_id::UniqueId;

/// Alias for the unique identifier of a task.
pub type TaskId = u64;

/// Struct to hold the state of a task.
///
/// # Fields
/// * `id` - The unique identifier of the task.
/// * `state` - The state of the task (running or paused).
/// * `handle` - The handle to the task.
pub struct Task {
    pub id: TaskId,
    pub state: Arc<AtomicBool>,
    pub handle: tokio::task::JoinHandle<()>,
}

/// Trait to implement on a type to schedule a task.
///
/// You must implement the `run` and `recurrence` methods.
pub trait ScheduleTask: UniqueId + Sized + Send + Sync + 'static {
    /// Start to run a task and schedule it to run periodically, based on the recurrence method.
    ///
    /// # Arguments
    /// * `is_running` - The state of the task (running or paused).
    ///
    /// # Returns
    /// A handle to the task.
    ///
    /// # Example
    /// ```rust,ignore
    /// let state = Arc::new(AtomicBool::new(true));
    /// let handle = MyTask.schedule(state.clone());
    /// ```
    fn schedule(self, is_running: Arc<AtomicBool>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if is_running.load(Ordering::Relaxed) {
                    self.run().await;
                }
                let _ = tokio::time::sleep(Self::recurrence()).await;
            }
        })
    }

    /// Run the task once and stop.
    fn run_once(self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    /// Define the recurrence of the task.
    ///
    /// The user must implement this method.
    fn recurrence() -> Duration;

    /// Function representing the task to run.
    ///
    /// The user must implement this method.
    fn run(&self) -> impl std::future::Future<Output = ()> + Send;
}
