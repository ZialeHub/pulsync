use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{recurrence::Recurrence, task::TaskId};

pub mod async_scheduler;
pub mod scheduler;

/// A type alias for the scheduler generic.
///
/// The scheduler is a hash map that stores the tasks scheduled.
/// The key is the unique identifier of the task and the value is the task itself.
/// Tasks can either be synchronous or asynchronous.
pub type Scheduler<T> = Arc<RwLock<HashMap<TaskId, T>>>;

/// A trait to implement on a scheduler.
///
/// Used to schedule, reschedule, pause, resume, and abort tasks.
pub trait TaskScheduler<T> {
    fn new() -> Self;
    fn schedule(&mut self, task: T, recurrence: Recurrence) -> Option<TaskId>;
    fn reschedule(&mut self, id: TaskId, recurrence: Recurrence) -> Option<TaskId>;
    fn pause(&mut self, id: TaskId);
    fn resume(&mut self, id: TaskId);
    fn abort(&mut self, id: TaskId);
    fn run(&self, task: T);
}
