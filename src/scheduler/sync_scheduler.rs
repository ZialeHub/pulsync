use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use chrono::NaiveDateTime;

use crate::{
    recurrence::Recurrence,
    scheduler::{Scheduler, TaskScheduler},
    task::{
        sync_task::{SyncTask, SyncTaskHandler},
        TaskId, TaskStatus,
    },
};

fn run_before_handler(
    id: TaskId,
    task: Box<dyn SyncTaskHandler + Send + Sync + 'static>,
    created_at: NaiveDateTime,
    tasks: Arc<RwLock<HashMap<TaskId, SyncTask>>>,
    status: Arc<RwLock<TaskStatus>>,
    recurrence: Arc<RwLock<Recurrence>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
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
}

fn run_after_handler(
    id: TaskId,
    task: Box<dyn SyncTaskHandler + Send + Sync + 'static>,
    created_at: NaiveDateTime,
    tasks: Arc<RwLock<HashMap<TaskId, SyncTask>>>,
    status: Arc<RwLock<TaskStatus>>,
    recurrence: Arc<RwLock<Recurrence>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            if *status.read().unwrap() == TaskStatus::Abort {
                break;
            }
            {
                let duration = {
                    let read_guard = recurrence.read().unwrap();
                    read_guard.unit
                };
                std::thread::sleep(duration.into());
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
        }
        tasks.write().unwrap().remove(&id);
    })
}

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
    ) -> Option<TaskId> {
        let id = task.unique_id(recurrence);
        if self.read().unwrap().contains_key(&id) {
            eprintln!("[schedule] Task ({id}): already exists");
            tracing::info!("[schedule] Task ({id}): already exists");
            return None;
        }
        let status = Arc::new(RwLock::new(TaskStatus::Running));
        let recurrence = Arc::new(RwLock::new(recurrence));
        let title = Arc::new(task.title());
        let created_at = chrono::Utc::now().naive_utc();
        let _handler = {
            let status = status.clone();
            let recurrence = recurrence.clone();
            let tasks = self.clone();
            match recurrence.clone().read().unwrap().run_after {
                true => run_after_handler(id, task, created_at, tasks, status, recurrence),
                false => run_before_handler(id, task, created_at, tasks, status, recurrence),
            }
        };
        let task = SyncTask {
            id,
            created_at,
            title,
            status,
            _handler,
            recurrence,
        };
        self.write().unwrap().insert(task.id, task);
        Some(id)
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
    fn reschedule(&mut self, id: TaskId, recurrence: Recurrence) -> Option<TaskId> {
        let mut binding = self.write().unwrap();
        let Some(task) = binding.get_mut(&id) else {
            eprintln!("[reschedule] Task ({id}): not found");
            tracing::info!("[reschedule] Task ({id}): not found");
            return None;
        };
        *task.recurrence.write().unwrap() = recurrence;
        Some(id)
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
    fn pause(&mut self, id: TaskId) {
        let mut binding = self.write().unwrap();
        let Some(task) = binding.get_mut(&id) else {
            eprintln!("[pause] Task ({id}): not found");
            tracing::info!("[pause] Task ({id}): not found");
            return;
        };
        if *task.status.read().unwrap() == TaskStatus::Paused {
            eprintln!("[pause] Task ({id}): already paused");
            tracing::info!("[pause] Task ({id}): already paused");
            return;
        }
        *task.status.write().unwrap() = TaskStatus::Paused;
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
    fn resume(&mut self, id: TaskId) {
        let mut binding = self.write().unwrap();
        let Some(task) = binding.get_mut(&id) else {
            eprintln!("[resume] Task ({id}): not found");
            tracing::info!("[resume] Task ({id}): not found");
            return;
        };
        if *task.status.read().unwrap() == TaskStatus::Running {
            eprintln!("[resume] Task ({id}): already resumed");
            tracing::info!("[resume] Task ({id}): already resumed");
            return;
        }
        *task.status.write().unwrap() = TaskStatus::Running;
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
    fn abort(&mut self, id: TaskId) {
        let mut binding = self.write().unwrap();
        let Some(task) = binding.remove(&id) else {
            eprintln!("[abort] Task ({id}): not found");
            tracing::info!("[abort] Task ({id}): not found");
            return;
        };
        *task.status.write().unwrap() = TaskStatus::Abort;
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
    fn get(&self) -> Vec<String> {
        self.read()
            .unwrap()
            .values()
            .map(|task| task.to_string())
            .collect()
    }
}
