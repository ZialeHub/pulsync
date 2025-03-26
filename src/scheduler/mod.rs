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

#[cfg(feature = "serde")]
impl serde::Serialize for Scheduler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct SerializedScheduler {
            pub saved_at: chrono::NaiveDateTime,
            pub tasks: Vec<(String, Recurrence)>,
        }

        let tasks: Vec<(String, Recurrence)> = self
            .read()
            .unwrap()
            .values()
            .map(|task| (String::new(), task.recurrence.read().unwrap().clone()))
            .collect();
        SerializedScheduler {
            saved_at: chrono::Local::now().naive_local(),
            tasks,
        }
        .serialize(serializer)
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
