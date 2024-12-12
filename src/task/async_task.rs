#[cfg(feature = "async")]
pub mod async_task {
    use std::{
        future::Future,
        sync::{atomic::AtomicBool, Arc, RwLock},
    };

    use crate::recurrence::Recurrence;

    use crate::task::{Task, TaskId};

    pub trait AsyncTaskHandler: Task {
        fn run(&self) -> impl Future<Output = ()> + Send;
    }

    pub struct AsyncTask<T: AsyncTaskHandler> {
        pub id: TaskId,
        pub state: Arc<AtomicBool>,
        pub handler: tokio::task::JoinHandle<()>,
        pub recurrence: Arc<RwLock<Recurrence>>,
        pub(crate) _phantom: std::marker::PhantomData<T>,
    }
    impl<T: AsyncTaskHandler> std::fmt::Debug for AsyncTask<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("AsyncTask")
                .field("id", &self.id)
                .field("state", &self.state)
                .finish()
        }
    }
}
