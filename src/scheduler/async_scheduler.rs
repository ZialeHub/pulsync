#[cfg(feature = "async")]
pub mod async_scheduler {
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
            async_task::async_task::{AsyncTask, AsyncTaskHandler},
            Task, TaskId,
        },
    };

    use crate::scheduler::{Scheduler, TaskScheduler};

    fn run_before_handler<T>(
        id: TaskId,
        tasks: Arc<RwLock<HashMap<TaskId, AsyncTask<T>>>>,
        state: Arc<RwLock<T>>,
        status: Arc<AtomicBool>,
        mut recurrence: Recurrence,
    ) -> tokio::task::JoinHandle<()>
    where
        T: Task + AsyncTaskHandler + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            loop {
                match recurrence.count {
                    None => {}
                    Some(count) if count == 0 => break,
                    Some(count) => {
                        recurrence.count = Some(count - 1);
                    }
                }
                if status.load(Ordering::Relaxed) {
                    state.read().unwrap().run();
                }
                let _ = tokio::time::sleep(recurrence.unit.into()).await;
            }
            tasks.write().unwrap().remove(&id);
        })
    }

    fn run_after_handler<T>(
        id: TaskId,
        tasks: Arc<RwLock<HashMap<TaskId, AsyncTask<T>>>>,
        state: Arc<RwLock<T>>,
        status: Arc<AtomicBool>,
        mut recurrence: Recurrence,
    ) -> tokio::task::JoinHandle<()>
    where
        T: Task + AsyncTaskHandler + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            loop {
                let _ = tokio::time::sleep(recurrence.unit.into()).await;
                match recurrence.count {
                    None => {}
                    Some(count) if count == 0 => break,
                    Some(count) => {
                        recurrence.count = Some(count - 1);
                    }
                }
                if status.load(Ordering::Relaxed) {
                    state.read().unwrap().run();
                }
            }
            tasks.write().unwrap().remove(&id);
        })
    }

    impl<T: AsyncTaskHandler> TaskScheduler<T> for Scheduler<AsyncTask<T>>
    where
        T: Task + AsyncTaskHandler + Send + Sync + 'static,
    {
        /// Create a new Asynchronous Scheduler.
        fn new() -> Self {
            Arc::new(RwLock::new(HashMap::new()))
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
        fn schedule(&mut self, task: T, recurrence: Recurrence) -> Option<TaskId> {
            let id = task.unique_id(recurrence);
            if self.read().unwrap().contains_key(&id) {
                eprintln!("[schedule] Task ({id}): already exists");
                tracing::info!("[schedule] Task ({id}): already exists");
                return None;
            }
            let status = Arc::new(AtomicBool::new(true));
            let state = Arc::new(RwLock::new(task));
            let handler = {
                let status = status.clone();
                let state = state.clone();
                match recurrence.run_after {
                    true => run_after_handler(id, self.clone(), state, status, recurrence),
                    false => run_before_handler(id, self.clone(), state, status, recurrence),
                }
            };
            let task = AsyncTask {
                id,
                created_at: chrono::Utc::now().naive_utc(),
                state,
                status,
                handler,
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
            let Some(task) = self.write().unwrap().remove(&id) else {
                eprintln!("[reschedule] Task ({id}): not found");
                tracing::info!("[reschedule] Task ({id}): not found");
                return None;
            };
            self.abort(id);
            let status = task.status.clone();
            let state = task.state.clone();
            let handler = {
                let status = status.clone();
                let state = state.clone();
                match recurrence.run_after {
                    true => run_after_handler(id, self.clone(), state, status, recurrence),
                    false => run_before_handler(id, self.clone(), state, status, recurrence),
                }
            };
            let task = AsyncTask {
                id,
                created_at: chrono::Utc::now().naive_utc(),
                state,
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
            task.handler.abort();
        }

        /// Run a task once.
        ///
        /// # Example
        /// ```rust,ignore
        /// scheduler.run(task);
        /// ```
        fn run(&self, task: T) {
            tokio::spawn(async move { task.run().await });
        }
    }
}
