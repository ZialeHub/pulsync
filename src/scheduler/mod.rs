use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[cfg(feature = "async")]
use crate::task::async_task::{AsyncTask, AsyncTaskHandler};
#[cfg(feature = "sync")]
use crate::task::sync_task::{SyncTask, SyncTaskHandler};
use crate::{
    error::PulsyncError,
    recurrence::Recurrence,
    task::{TaskDetails, TaskId},
};

#[cfg(feature = "async")]
mod async_scheduler;
#[cfg(feature = "sync")]
mod sync_scheduler;

/// A type alias for the scheduler generic.
///
/// The scheduler is a hash map that stores the tasks scheduled.
/// The key is the unique identifier of the task and the value is the task itself.
/// Tasks can either be synchronous or asynchronous.
#[cfg(feature = "sync")]
pub struct Scheduler(Arc<RwLock<HashMap<TaskId, SyncTask>>>);
#[cfg(feature = "async")]
pub struct Scheduler(Arc<RwLock<HashMap<TaskId, AsyncTask>>>);

#[cfg(feature = "sync")]
impl std::ops::Deref for Scheduler {
    type Target = Arc<RwLock<HashMap<TaskId, SyncTask>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "async")]
impl std::ops::Deref for Scheduler {
    type Target = Arc<RwLock<HashMap<TaskId, AsyncTask>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "sync")]
impl std::ops::DerefMut for Scheduler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "async")]
impl std::ops::DerefMut for Scheduler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A trait to implement on a scheduler.
///
/// Used to schedule, reschedule, pause, resume, and abort tasks.
pub trait TaskScheduler {
    fn build() -> Self;
    #[cfg(feature = "sync")]
    fn schedule(
        &mut self,
        task: Box<dyn SyncTaskHandler + Send + Sync + 'static>,
        recurrence: Recurrence,
    ) -> Result<TaskId, PulsyncError>;
    #[cfg(feature = "async")]
    fn schedule(
        &mut self,
        task: Box<dyn AsyncTaskHandler + Send + Sync + 'static>,
        recurrence: Recurrence,
    ) -> Result<TaskId, PulsyncError>;
    fn reschedule(&mut self, id: TaskId, recurrence: Recurrence) -> Result<TaskId, PulsyncError>;
    fn pause(&mut self, id: TaskId) -> Result<(), PulsyncError>;
    fn resume(&mut self, id: TaskId) -> Result<(), PulsyncError>;
    fn abort(&mut self, id: TaskId) -> Result<(), PulsyncError>;
    #[cfg(feature = "sync")]
    fn run(&self, task: impl SyncTaskHandler);
    #[cfg(feature = "async")]
    fn run(&self, task: impl AsyncTaskHandler);
    fn get(&self) -> Vec<TaskDetails>;
    #[cfg(all(feature = "sync", feature = "serde"))]
    fn restart(
        tasks: Vec<(Box<dyn SyncTaskHandler + Send + Sync + 'static>, Recurrence)>,
    ) -> (Self, Vec<Option<TaskId>>);
    #[cfg(all(feature = "async", feature = "serde"))]
    fn restart(
        tasks: Vec<(
            Box<dyn AsyncTaskHandler + Send + Sync + 'static>,
            Recurrence,
        )>,
    ) -> (Self, Vec<Option<TaskId>>);
}

#[cfg(feature = "sync")]
impl<T: SyncTaskHandler + Send + Sync> From<T> for Box<dyn SyncTaskHandler + Send + Sync> {
    fn from(value: T) -> Self {
        Box::new(value) as Box<dyn SyncTaskHandler + Send + Sync>
    }
}

#[cfg(feature = "async")]
impl<T: AsyncTaskHandler + Send + Sync> From<T> for Box<dyn AsyncTaskHandler + Send + Sync> {
    fn from(value: T) -> Self {
        Box::new(value) as Box<dyn AsyncTaskHandler + Send + Sync>
    }
}
