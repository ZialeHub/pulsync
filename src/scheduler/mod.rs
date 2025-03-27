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
    fn abort(&mut self, id: TaskId);
    #[cfg(feature = "sync")]
    fn run(&self, task: impl SyncTaskHandler);
    #[cfg(feature = "async")]
    fn run(&self, task: impl AsyncTaskHandler);
    fn get(&self) -> Vec<String>;
}

#[cfg(feature = "serde")]
pub trait Restart
where
    Self: Sized,
{
    #[cfg(feature = "sync")]
    fn restart(
        tasks: Vec<(Box<dyn SyncTaskHandler + Send + Sync + 'static>, Recurrence)>,
    ) -> (Self, Vec<Option<TaskId>>);
    #[cfg(feature = "async")]
    fn restart(
        tasks: Vec<(
            Box<dyn AsyncTaskHandler + Send + Sync + 'static>,
            Recurrence,
        )>,
    ) -> (Self, Vec<Option<TaskId>>);
}

#[cfg(feature = "serde")]
impl Restart for Scheduler {
    #[cfg(feature = "sync")]
    fn restart(
        tasks: Vec<(Box<dyn SyncTaskHandler + Send + Sync + 'static>, Recurrence)>,
    ) -> (Self, Vec<Option<TaskId>>) {
        let mut scheduler = Self::build();
        let mut task_ids = Vec::new();
        for (task, recurrence) in tasks.into_iter() {
            let new_id = scheduler.schedule(task, recurrence);
            task_ids.push(new_id);
        }
        (scheduler, task_ids)
    }
    #[cfg(feature = "async")]
    fn restart(
        tasks: Vec<(
            Box<dyn AsyncTaskHandler + Send + Sync + 'static>,
            Recurrence,
        )>,
    ) -> (Self, Vec<Option<TaskId>>) {
        let mut scheduler = Self::build();
        let mut task_ids = Vec::new();
        for (task, recurrence) in tasks.into_iter() {
            let new_id = scheduler.schedule(task, recurrence);
            task_ids.push(new_id);
        }
        (scheduler, task_ids)
    }
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
