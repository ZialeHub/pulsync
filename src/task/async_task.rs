#[cfg(feature = "async")]
pub mod async_task {
    use std::{
        future::Future,
        sync::{atomic::AtomicBool, Arc, RwLock},
    };

    use chrono::NaiveDateTime;

    use crate::recurrence::Recurrence;

    use crate::task::{Task, TaskId};

    /// A trait for handling asynchronous tasks.
    pub trait AsyncTaskHandler: Task {
        fn run(&self) -> impl Future<Output = ()> + Send;
    }

    /// Represents an asynchronous task, scheduled in the scheduler.
    ///
    /// # Fields
    /// * `id` - The unique identifier of the task.
    /// * `state` - The state of the task (to pause/resume the execution).
    /// * `handler` - The handler of the task.
    /// * `recurrence` - The recurrence of the task.
    pub struct AsyncTask<T: AsyncTaskHandler> {
        pub id: TaskId,
        pub created_at: NaiveDateTime,
        pub state: Arc<RwLock<T>>,
        pub status: Arc<AtomicBool>,
        pub handler: tokio::task::JoinHandle<()>,
        pub recurrence: Recurrence,
    }

    impl<T: AsyncTaskHandler> std::fmt::Display for AsyncTask<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "[AsyncTask({})] {} started at {}",
                self.id,
                self.state.read().unwrap().title(),
                self.created_at.date().to_string()
            )
        }
    }
}
