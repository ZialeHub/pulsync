use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    error::PulsyncError,
    recurrence::Recurrence,
    scheduler::{Scheduler, TaskScheduler},
    task::{
        sync_task::{SyncTask, SyncTaskHandler},
        TaskDetails, TaskId, TaskStatus,
    },
};

impl TaskScheduler for Scheduler {
    /// Create a new Asynchronous Scheduler.
    fn build() -> Self {
        Scheduler(Arc::new(RwLock::new(HashMap::<TaskId, SyncTask>::new())))
    }

    /// Add a task to the scheduler.
    ///
    /// The task will be executed every `recurrence` time (according to the `unit` and `count`).
    ///
    /// Do nothing if the task already exists in the scheduler.
    ///
    /// # Example
    /// ```rust,ignore
    /// let id = scheduler.schedule(task, every(1.seconds())).unwrap();
    /// ```
    fn schedule(
        &mut self,
        task: Box<dyn SyncTaskHandler + Send + Sync + 'static>,
        recurrence: Recurrence,
    ) -> Result<TaskId, PulsyncError> {
        let id = task.unique_id(recurrence);
        if self.read().unwrap().contains_key(&id) {
            tracing::info!("[schedule] Task ({id}): already scheduled");
            return Err(PulsyncError::AlreadyScheduled(id));
        }
        let status = Arc::new(RwLock::new(TaskStatus::Running));
        let recurrence = Arc::new(RwLock::new(recurrence));
        let title = Arc::new(task.title());
        let created_at = chrono::Utc::now().naive_utc();
        let _handle = {
            let status = status.clone();
            let recurrence = recurrence.clone();
            let tasks = self.clone();
            std::thread::spawn(move || {
                {
                    if recurrence.clone().read().unwrap().run_after {
                        let duration = {
                            let read_guard = recurrence.read().unwrap();
                            read_guard.unit
                        };
                        std::thread::sleep(duration.into());
                    }
                }
                loop {
                    if *status.read().unwrap() == TaskStatus::Abort {
                        break;
                    }
                    {
                        let mut recurrence = recurrence.write().unwrap();
                        if let Some(limit) = recurrence.limit {
                            let now = chrono::Utc::now().naive_utc();
                            let elapsed_time = now - created_at;
                            if elapsed_time.num_seconds() as u64 >= *limit {
                                break;
                            }
                        }
                        if let Some(limit_datetime) = recurrence.limit_datetime {
                            if limit_datetime <= chrono::Utc::now().naive_utc() {
                                break;
                            }
                        }
                        match recurrence.count {
                            None => {}
                            Some(0) => break,
                            Some(count) => {
                                recurrence.count = Some(count - 1);
                            }
                        }
                    }
                    if *status.read().unwrap() == TaskStatus::Running {
                        task.run();
                    }
                    {
                        let duration = {
                            let read_guard = recurrence.read().unwrap();
                            read_guard.unit
                        };
                        std::thread::sleep(duration.into());
                    }
                }
                tasks.write().unwrap().remove(&id);
            })
        };
        let task = SyncTask {
            id,
            created_at,
            title,
            status,
            _handle,
            recurrence,
        };
        self.write().unwrap().insert(task.id, task);
        Ok(id)
    }

    /// Update the recurrence of a task.
    ///
    /// Do nothing if the task does not exist in the scheduler.
    ///
    /// # Example
    /// ```rust,ignore
    /// let id = scheduler.schedule(task, every(1.seconds())).unwrap();
    /// let new_id = scheduler.reschedule(id, every(3.seconds())).unwrap();
    /// ```
    fn reschedule(&mut self, id: TaskId, recurrence: Recurrence) -> Result<TaskId, PulsyncError> {
        let mut binding = self.write().unwrap();
        let Some(task) = binding.get_mut(&id) else {
            tracing::info!("[reschedule] Task ({id}): not found");
            return Err(PulsyncError::TaskNotFound("reschedule", id));
        };
        *task.recurrence.write().unwrap() = recurrence;
        Ok(id)
    }

    /// Pause the execution of a task.
    ///
    /// Do nothing if the task doesn't exist in the scheduler or if the task is already paused.
    ///
    /// # Example
    /// ```rust,ignore
    /// let id = scheduler.schedule(task, every(1.seconds())).unwrap();
    /// scheduler.pause(id);
    /// ```
    fn pause(&mut self, id: TaskId) -> Result<(), PulsyncError> {
        let mut binding = self.write().unwrap();
        let Some(task) = binding.get_mut(&id) else {
            tracing::info!("[pause] Task ({id}): not found");
            return Err(PulsyncError::TaskNotFound("pause", id));
        };
        if *task.status.read().unwrap() == TaskStatus::Paused {
            tracing::info!("[pause] Task ({id}): already paused");
            return Err(PulsyncError::AlreadyPaused(id));
        }
        *task.status.write().unwrap() = TaskStatus::Paused;
        Ok(())
    }

    /// Resume the execution of a task.
    ///
    /// Do nothing if the task doesn't exist in the scheduler or if the task is already resumed.
    ///
    /// # Example
    /// ```rust,ignore
    /// let id = scheduler.schedule(task, every(1.seconds())).unwrap();
    /// scheduler.resume(id);
    /// ```
    fn resume(&mut self, id: TaskId) -> Result<(), PulsyncError> {
        let mut binding = self.write().unwrap();
        let Some(task) = binding.get_mut(&id) else {
            tracing::info!("[resume] Task ({id}): not found");
            return Err(PulsyncError::TaskNotFound("resume", id));
        };
        if *task.status.read().unwrap() == TaskStatus::Running {
            tracing::info!("[resume] Task ({id}): already resumed");
            return Err(PulsyncError::AlreadyResumed(id));
        }
        *task.status.write().unwrap() = TaskStatus::Running;
        Ok(())
    }

    /// Abort the execution of a task. (The task will be removed from the scheduler)
    ///
    /// Do nothing if the task doesn't exist in the scheduler.
    ///
    /// # Example
    /// ```rust,ignore
    /// let id = scheduler.schedule(task, every(1.seconds())).unwrap();
    /// scheduler.abort(id);
    /// ```
    fn abort(&mut self, id: TaskId) -> Result<(), PulsyncError> {
        let mut binding = self.write().unwrap();
        let Some(task) = binding.remove(&id) else {
            tracing::info!("[abort] Task ({id}): not found");
            return Err(PulsyncError::TaskNotFound("abort", id));
        };
        *task.status.write().unwrap() = TaskStatus::Abort;
        Ok(())
    }

    /// Run a task once.
    ///
    /// # Example
    /// ```rust,ignore
    /// scheduler.run(task);
    /// ```
    fn run(&self, task: impl SyncTaskHandler) {
        std::thread::spawn(move || task.run());
    }

    /// Collect the title of all tasks in the scheduler.
    ///
    /// # Example
    /// ```rust,ignore
    /// let titles = scheduler.get();
    /// ```
    fn get(&self) -> Vec<TaskDetails> {
        self.read()
            .unwrap()
            .values()
            .map(|task| TaskDetails {
                id: task.id,
                label: task.to_string(),
                recurrence: task.recurrence.clone(),
                status: *task.status.read().unwrap(),
                created_at: task.created_at,
            })
            .collect()
    }

    #[cfg(feature = "serde")]
    fn restart(
        tasks: Vec<(Box<dyn SyncTaskHandler + Send + Sync + 'static>, Recurrence)>,
    ) -> (Self, Vec<Option<TaskId>>) {
        let mut scheduler = Self::build();
        let task_ids = tasks
            .into_iter()
            .map(|(task, recurrence)| scheduler.schedule(task, recurrence))
            .collect::<Vec<Result<TaskId, PulsyncError>>>();
        (scheduler, task_ids)
    }
}
