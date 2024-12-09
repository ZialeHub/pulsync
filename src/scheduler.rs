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

#[cfg(test)]
mod test {
    use std::{
        sync::{Arc, RwLock},
        time::Duration,
    };

    use crate::prelude::*;

    #[tokio::test]
    async fn start_and_stop_task() {
        struct TestTask {
            state: Arc<RwLock<u32>>,
        }
        impl UniqueId for TestTask {}
        impl ScheduleTask for TestTask {
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }

            fn recurrence() -> Duration {
                Duration::from_secs(1)
            }
        }

        let mut scheduler = Scheduler::new();
        let value = Arc::new(RwLock::new(0));
        let task = TestTask {
            state: value.clone(),
        };
        scheduler.start(task);
        tokio::time::sleep(Duration::from_secs(2)).await;
        scheduler.stop(TestTask::get_unique_id());
        assert_eq!(*value.read().unwrap(), 2);
    }

    #[tokio::test]
    async fn start_2secs_pause_2secs_resume_2secs_stop_task() {
        struct TestTask {
            state: Arc<RwLock<u32>>,
        }
        impl UniqueId for TestTask {}
        impl ScheduleTask for TestTask {
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }

            fn recurrence() -> Duration {
                Duration::from_secs(1)
            }
        }

        let mut scheduler = Scheduler::new();
        let value = Arc::new(RwLock::new(0));
        let task = TestTask {
            state: value.clone(),
        };
        scheduler.start(task);
        tokio::time::sleep(Duration::from_secs(2)).await;
        scheduler.pause(TestTask::get_unique_id());
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(*value.read().unwrap(), 2);
        scheduler.resume(TestTask::get_unique_id());
        tokio::time::sleep(Duration::from_secs(2)).await;
        scheduler.stop(TestTask::get_unique_id());
        assert_eq!(*value.read().unwrap(), 4);
    }

    #[tokio::test]
    async fn start_and_stop_multiple_tasks() {
        struct FirstTask {
            state: Arc<RwLock<u32>>,
        }
        impl UniqueId for FirstTask {}
        impl ScheduleTask for FirstTask {
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }

            fn recurrence() -> Duration {
                Duration::from_secs(1)
            }
        }

        struct SecondTask {
            state: Arc<RwLock<u32>>,
        }
        impl UniqueId for SecondTask {}
        impl ScheduleTask for SecondTask {
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }

            fn recurrence() -> Duration {
                Duration::from_secs(2)
            }
        }

        let mut scheduler = Scheduler::new();
        let first_value = Arc::new(RwLock::new(0));
        let first_task = FirstTask {
            state: first_value.clone(),
        };
        let second_value = Arc::new(RwLock::new(0));
        let second_task = SecondTask {
            state: second_value.clone(),
        };
        scheduler.start(first_task);
        scheduler.start(second_task);
        tokio::time::sleep(Duration::from_secs(5)).await;
        scheduler.stop(FirstTask::get_unique_id());
        scheduler.stop(SecondTask::get_unique_id());
        assert_eq!(*first_value.read().unwrap(), 5);
        assert_eq!(*second_value.read().unwrap(), 3);
    }

    #[tokio::test]
    async fn start_2secs_pause_2secs_resume_2secs_stop_multiple_tasks() {
        struct FirstTask {
            state: Arc<RwLock<u32>>,
        }
        impl UniqueId for FirstTask {}
        impl ScheduleTask for FirstTask {
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }

            fn recurrence() -> Duration {
                Duration::from_secs(1)
            }
        }

        struct SecondTask {
            state: Arc<RwLock<u32>>,
        }
        impl UniqueId for SecondTask {}
        impl ScheduleTask for SecondTask {
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }

            fn recurrence() -> Duration {
                Duration::from_secs(2)
            }
        }

        let mut scheduler = Scheduler::new();
        let first_value = Arc::new(RwLock::new(0));
        let first_task = FirstTask {
            state: first_value.clone(),
        };
        let second_value = Arc::new(RwLock::new(0));
        let second_task = SecondTask {
            state: second_value.clone(),
        };
        scheduler.start(first_task);
        scheduler.start(second_task);
        tokio::time::sleep(Duration::from_secs(2)).await;
        scheduler.pause(FirstTask::get_unique_id());
        scheduler.pause(SecondTask::get_unique_id());
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(*first_value.read().unwrap(), 2);
        assert_eq!(*second_value.read().unwrap(), 1);
        scheduler.resume(FirstTask::get_unique_id());
        scheduler.resume(SecondTask::get_unique_id());
        tokio::time::sleep(Duration::from_secs(2)).await;
        scheduler.stop(FirstTask::get_unique_id());
        scheduler.stop(SecondTask::get_unique_id());
        assert_eq!(*first_value.read().unwrap(), 4);
        assert_eq!(*second_value.read().unwrap(), 2);
    }
}
