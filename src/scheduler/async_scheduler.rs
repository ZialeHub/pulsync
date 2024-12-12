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
            Task, UniqueId,
        },
    };

    use crate::scheduler::{Scheduler, TaskScheduler};

    impl<T: AsyncTaskHandler> TaskScheduler<T> for Scheduler<AsyncTask<T>>
    where
        T: Task + AsyncTaskHandler + Send + Sync + 'static,
    {
        fn new() -> Self {
            HashMap::new()
        }

        fn schedule(&mut self, task: T, recurrence: Recurrence) {
            let id = T::unique_id();
            if self.contains_key(&id) {
                eprintln!("Task ({id}): already exists");
                tracing::info!("Task ({id}): already exists");
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
                            task.run().await;
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
            let task = AsyncTask {
                id,
                state,
                handler,
                recurrence,
                _phantom: std::marker::PhantomData,
            };
            self.insert(task.id, task);
        }

        fn pause<U: UniqueId>(&mut self) {
            // Pause the task
            let id = U::unique_id();
            let Some(task) = self.get_mut(&U::unique_id()) else {
                eprintln!("Task ({id}): not found");
                tracing::info!("Task ({id}): not found");
                return;
            };
            if !task.state.load(Ordering::Relaxed) {
                eprintln!("Task ({id}): already paused");
                tracing::info!("Task ({id}): already paused");
                return;
            }
            task.state.store(false, Ordering::Relaxed);
        }

        fn resume<U: UniqueId>(&mut self) {
            // Resume the task
            let id = U::unique_id();
            let Some(task) = self.get_mut(&T::unique_id()) else {
                eprintln!("Task ({id}): not found");
                tracing::info!("Task ({id}): not found");
                return;
            };
            if task.state.load(Ordering::Relaxed) {
                eprintln!("Task ({id}): already resumed");
                tracing::info!("Task ({id}): already resumed");
                return;
            }
            task.state.store(true, Ordering::Relaxed);
        }

        fn abort<U: UniqueId>(&mut self) {
            // Abort the task
            let id = U::unique_id();
            let Some(task) = self.remove(&id) else {
                eprintln!("Task ({id}): not found");
                tracing::info!("Task ({id}): not found");
                return;
            };
            task.handler.abort();
        }

        fn run(&self, task: T) {
            // Run task once
            tokio::spawn(async move { task.run().await });
        }
    }
}
