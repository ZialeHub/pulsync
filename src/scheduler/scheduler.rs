#[cfg(feature = "sync")]
pub mod sync_scheduler {
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
            task::sync_task::{SyncTask, SyncTaskHandler},
            Task, UniqueId,
        },
    };

    use crate::scheduler::{Scheduler, TaskScheduler};

    impl<T: SyncTaskHandler> TaskScheduler<T> for Scheduler<SyncTask<T>>
    where
        T: Task + SyncTaskHandler + Send + Sync + 'static,
    {
        /// Create a new Synchonous Scheduler.
        fn new() -> Self {
            HashMap::new()
        }

        /// Add a task to the scheduler.
        ///
        /// The task will be executed every `recurrence` time (according to the `unit` and `count`).
        ///
        /// Do nothing if the task already exists in the scheduler.
        ///
        /// # Example
        /// ```rust,ignore
        /// scheduler.schedule(task, every(1.seconds()));
        /// ```
        fn schedule(&mut self, task: T, recurrence: Recurrence) {
            let id = T::unique_id();
            if self.contains_key(&id) {
                eprintln!("[schedule] Task ({id}): already exists");
                tracing::info!("[schedule] Task ({id}): already exists");
                return;
            }
            let state = Arc::new(AtomicBool::new(true));
            let recurrence = Arc::new(RwLock::new(recurrence));
            let handler = {
                let state = state.clone();
                let recurrence = recurrence.clone();
                tokio::spawn(async move {
                    loop {
                        {
                            let mut recurrence = recurrence.write().unwrap();
                            match recurrence.count {
                                None => {}
                                Some(count) if count == 0 => break,
                                Some(count) => {
                                    recurrence.count = Some(count - 1);
                                }
                            }
                        }
                        if state.load(Ordering::Relaxed) {
                            task.run();
                        }
                        {
                            let duration = {
                                let read_guard = recurrence.read().unwrap();
                                read_guard.unit
                            };
                            let _ = tokio::time::sleep(duration.into()).await;
                        }
                    }
                })
            };
            let task = SyncTask {
                id,
                state,
                handler,
                recurrence,
                _phantom: std::marker::PhantomData,
            };
            self.insert(task.id, task);
        }

        /// Update the recurrence of a task.
        ///
        /// Do nothing if the task does not exist in the scheduler.
        ///
        /// # Example
        /// ```rust,ignore
        /// scheduler.schedule(task, every(1.seconds()));
        /// scheduler.reschedule::<MySyncTask>(every(3.seconds()));
        /// ```
        fn reschedule<U: UniqueId>(&mut self, recurrence: Recurrence) {
            let id = U::unique_id();
            let Some(task) = self.get_mut(&id) else {
                eprintln!("[reschedule] Task ({id}): not found");
                tracing::info!("[reschedule] Task ({id}): not found");
                return;
            };
            let mut current_recurrence = task.recurrence.write().unwrap();
            *current_recurrence = recurrence;
        }

        /// Pause the execution of a task.
        ///
        /// Do nothing if the task doesn't exist in the scheduler or if the task is already paused.
        ///
        /// # Example
        /// ```rust,ignore
        /// scheduler.pause::<MySyncTask>();
        /// ```
        fn pause<U: UniqueId>(&mut self) {
            let id = U::unique_id();
            let Some(task) = self.get_mut(&id) else {
                eprintln!("[pause] Task ({id}): not found");
                tracing::info!("[pause] Task ({id}): not found");
                return;
            };
            if !task.state.load(Ordering::Relaxed) {
                eprintln!("[pause] Task ({id}): already paused");
                tracing::info!("[pause] Task ({id}): already paused");
                return;
            }
            task.state.store(false, Ordering::Relaxed);
        }

        /// Resume the execution of a task.
        ///
        /// Do nothing if the task doesn't exist in the scheduler or if the task is already resumed.
        ///
        /// # Example
        /// ```rust,ignore
        /// scheduler.resume::<MySyncTask>();
        /// ```
        fn resume<U: UniqueId>(&mut self) {
            let id = U::unique_id();
            let Some(task) = self.get_mut(&id) else {
                eprintln!("[resume] Task ({id}): not found");
                tracing::info!("[resume] Task ({id}): not found");
                return;
            };
            if task.state.load(Ordering::Relaxed) {
                eprintln!("[resume] Task ({id}): already resumed");
                tracing::info!("[resume] Task ({id}): already resumed");
                return;
            }
            task.state.store(true, Ordering::Relaxed);
        }

        /// Abort the execution of a task. (The task will be removed from the scheduler)
        ///
        /// Do nothing if the task doesn't exist in the scheduler.
        ///
        /// # Example
        /// ```rust,ignore
        /// scheduler.abort::<MySyncTask>();
        /// ```
        fn abort<U: UniqueId>(&mut self) {
            let id = U::unique_id();
            let Some(task) = self.remove(&id) else {
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
            tokio::spawn(async move { task.run() });
        }
    }
}
