#[cfg(feature = "async")]
pub mod async_task {
    use std::{
        future::Future,
        pin::Pin,
        sync::{atomic::AtomicBool, Arc, RwLock},
    };

    use chrono::NaiveDateTime;

    use crate::recurrence::Recurrence;

    use crate::task::{Task, TaskId};

    /// A trait for handling asynchronous tasks.
    pub trait AsyncTaskHandler: Task + Send {
        fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>>;
    }

    /// Represents an asynchronous task, scheduled in the scheduler.
    ///
    /// # Fields
    /// * `id` - The unique identifier of the task.
    /// * `state` - The state of the task (to pause/resume the execution).
    /// * `handler` - The handler of the task.
    /// * `recurrence` - The recurrence of the task.
    pub struct AsyncTask {
        pub id: TaskId,
        pub created_at: NaiveDateTime,
        //pub state: Arc<RwLock<Box<dyn AsyncTaskHandler + Send + 'static + Sync>>>,
        pub status: Arc<AtomicBool>,
        pub title: Arc<String>,
        pub handler: tokio::task::JoinHandle<()>,
        pub recurrence: Arc<RwLock<Recurrence>>,
        //pub _phantom: std::marker::PhantomData<T>,
    }

    impl std::fmt::Display for AsyncTask {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "[AsyncTask({})] {} started at {}",
                self.id,
                self.title,
                self.created_at.date().to_string()
            )
        }
    }
}
