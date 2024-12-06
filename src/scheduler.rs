use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::task::{ScheduleTask, Task, TaskId};

/// Struct to hold every task scheduled.
#[derive(Default)]
pub struct Scheduler {
    pub tasks: HashMap<TaskId, Task>,
}
impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a task and schedule it to run periodically.
    ///
    /// Do nothing if the task is already scheduled. (based on the unique identifier)
    ///
    /// # Arguments
    /// * `task` - The task to start.
    ///
    /// # Example
    ///
    /// This task will print
    /// ```rust,ignore
    /// struct MyTask;
    /// impl UniqueId for MyTask {}
    /// impl ScheduleTask for MyTask {
    ///     async fn run(&self) {
    ///         println!("Hello, world!");
    ///     }
    ///
    ///     fn recurrence() -> Duration {
    ///         Duration::from_secs(5)
    ///     }
    /// }
    ///
    /// let mut scheduler = Scheduler::new();
    /// scheduler.start(MyTask);
    /// ```
    pub fn start<T: ScheduleTask>(&mut self, task: T) {
        let id = T::get_unique_id();
        if self.tasks.contains_key(&id) {
            tracing::info!("Task ({id}): already exists");
            return;
        }
        let state = Arc::new(AtomicBool::new(true));
        let task = Task {
            id,
            handle: task.schedule(state.clone()),
            state,
        };
        self.tasks.insert(id, task);
    }

    /// Remove a task from the scheduler.
    ///
    /// Do nothing if the task is not found.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the task.
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut scheduler = Scheduler::new();
    /// scheduler.stop(MyTask::get_unique_id());
    /// ```
    pub fn stop(&mut self, id: TaskId) {
        let Some(task) = self.tasks.remove(&id) else {
            tracing::info!("Task ({id}): not found");
            return;
        };
        task.handle.abort();
    }

    /// Pause a task from the scheduler.
    ///
    /// Do nothing if the task is not found or already paused.
    ///
    /// The task will not run until it is resumed.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the task.
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut scheduler = Scheduler::new();
    /// scheduler.pause(MyTask::get_unique_id());
    /// ```
    pub fn pause(&mut self, id: TaskId) {
        let Some(task) = self.tasks.get_mut(&id) else {
            tracing::info!("Task ({id}): not found");
            return;
        };
        if !task.state.load(Ordering::Relaxed) {
            tracing::info!("Task ({id}): already paused");
            return;
        }
        task.state.store(false, Ordering::Relaxed);
    }

    /// Resume a task from the scheduler.
    ///
    /// Do nothing if the task is not found or already resumed.
    ///
    /// The task will run again periodically based on the recurrence method.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the task.
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut scheduler = Scheduler::new();
    /// scheduler.resume(MyTask::get_unique_id());
    /// ```
    pub fn resume(&mut self, id: TaskId) {
        let Some(task) = self.tasks.get_mut(&id) else {
            tracing::info!("Task ({id}): not found");
            return;
        };
        if task.state.load(Ordering::Relaxed) {
            tracing::info!("Task ({id}): already resumed");
            return;
        }
        task.state.store(true, Ordering::Relaxed);
    }
}
