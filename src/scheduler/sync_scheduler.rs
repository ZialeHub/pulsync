use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use crate::{
    recurrence::Recurrence,
    task::{
        sync_task::{SyncTask, SyncTaskHandler},
        TaskId,
    },
};

use crate::scheduler::{Scheduler, TaskScheduler};

fn run_before_handler(
    id: TaskId,
    task: Box<dyn SyncTaskHandler + Send + Sync + 'static>,
    tasks: Arc<RwLock<HashMap<TaskId, SyncTask>>>,
    status: Arc<AtomicBool>,
    recurrence: Arc<RwLock<Recurrence>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            {
                let mut recurrence = recurrence.write().unwrap();
                match recurrence.count {
                    None => {}
                    Some(0) => break,
                    Some(count) => {
                        recurrence.count = Some(count - 1);
                    }
                }
            }
            if status.load(Ordering::Relaxed) {
                task.run();
            }
            {
                let duration = {
                    let read_guard = recurrence.read().unwrap();
                    read_guard.unit
                };
                let _ = std::thread::sleep(duration.into());
            }
        }
        tasks.write().unwrap().remove(&id);
    })
}

fn run_after_handler(
    id: TaskId,
    task: Box<dyn SyncTaskHandler + Send + Sync + 'static>,
    tasks: Arc<RwLock<HashMap<TaskId, SyncTask>>>,
    status: Arc<AtomicBool>,
    recurrence: Arc<RwLock<Recurrence>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            {
                let duration = {
                    let read_guard = recurrence.read().unwrap();
                    read_guard.unit
                };
                let _ = std::thread::sleep(duration.into());
            }
            {
                let mut recurrence = recurrence.write().unwrap();
                match recurrence.count {
                    None => {}
                    Some(0) => break,
                    Some(count) => {
                        recurrence.count = Some(count - 1);
                    }
                }
            }
            if status.load(Ordering::Relaxed) {
                task.run();
            }
        }
        tasks.write().unwrap().remove(&id);
    })
}

impl TaskScheduler for Scheduler {
    /// Create a new Asynchronous Scheduler.
    fn build() -> Self {
        Arc::new(RwLock::new(HashMap::<TaskId, SyncTask>::new()))
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
        let status = Arc::new(AtomicBool::new(true));
        let recurrence = Arc::new(RwLock::new(recurrence));
        let title = Arc::new(task.title());
        let handler = {
            let status = status.clone();
            let recurrence = recurrence.clone();
            let tasks = self.clone();
            match recurrence.clone().read().unwrap().run_after {
                true => run_after_handler(id, task, tasks, status, recurrence),
                false => run_before_handler(id, task, tasks, status, recurrence),
            }
        };
        let task = SyncTask {
            id,
            created_at: chrono::Utc::now().naive_utc(),
            title,
            status,
            handler,
            recurrence,
        };
        self.write().unwrap().insert(task.id, task);
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
        if !task.status.load(Ordering::Relaxed) {
            eprintln!("[pause] Task ({id}): already paused");
            tracing::info!("[pause] Task ({id}): already paused");
            return;
        }
        task.status.store(false, Ordering::Relaxed);
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
        if task.status.load(Ordering::Relaxed) {
            eprintln!("[resume] Task ({id}): already resumed");
            tracing::info!("[resume] Task ({id}): already resumed");
            return;
        }
        task.status.store(true, Ordering::Relaxed);
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

    fn get(&self) -> Vec<String> {
        self.read()
            .unwrap()
            .values()
            .map(|task| task.to_string())
            .collect()
    }
}
