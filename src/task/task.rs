#[cfg(feature = "sync")]
pub mod sync_task {
    use std::sync::{atomic::AtomicBool, Arc, RwLock};

    use crate::recurrence::Recurrence;

    use crate::task::{Task, TaskId};

    pub trait SyncTaskHandler: Task {
        fn run(&self);
    }

    pub struct SyncTask<T: SyncTaskHandler> {
        pub id: TaskId,
        pub state: Arc<AtomicBool>,
        pub handler: tokio::task::JoinHandle<()>,
        pub recurrence: Arc<RwLock<Recurrence>>,
        pub(crate) _phantom: std::marker::PhantomData<T>,
    }
    impl<T: SyncTaskHandler> std::fmt::Debug for SyncTask<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SyncTask")
                .field("id", &self.id)
                .field("state", &self.state)
                .finish()
        }
    }
}
