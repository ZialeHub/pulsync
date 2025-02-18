use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[cfg(feature = "async")]
use crate::task::async_task::{AsyncTask, AsyncTaskHandler};
#[cfg(feature = "sync")]
use crate::task::sync_task::{SyncTask, SyncTaskHandler};
use crate::{recurrence::Recurrence, task::TaskId};

#[cfg(feature = "async")]
pub mod async_scheduler;
#[cfg(feature = "sync")]
pub mod sync_scheduler;

/// A type alias for the scheduler generic.
///
/// The scheduler is a hash map that stores the tasks scheduled.
/// The key is the unique identifier of the task and the value is the task itself.
/// Tasks can either be synchronous or asynchronous.
#[cfg(feature = "sync")]
pub type Scheduler = Arc<RwLock<HashMap<TaskId, SyncTask>>>;
#[cfg(feature = "async")]
pub type Scheduler = Arc<RwLock<HashMap<TaskId, AsyncTask>>>;

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
    ) -> Option<TaskId>;
    #[cfg(feature = "async")]
    fn schedule(
        &mut self,
        task: Box<dyn AsyncTaskHandler + Send + Sync + 'static>,
        recurrence: Recurrence,
    ) -> Option<TaskId>;
    fn reschedule(&mut self, id: TaskId, recurrence: Recurrence) -> Option<TaskId>;
    fn pause(&mut self, id: TaskId);
    fn resume(&mut self, id: TaskId);
    #[cfg(feature = "async")]
    fn abort(&mut self, id: TaskId);
    #[cfg(feature = "sync")]
    fn run(&self, task: impl SyncTaskHandler);
    #[cfg(feature = "async")]
    fn run(&self, task: impl AsyncTaskHandler);
    fn get(&self) -> Vec<String>;
}
