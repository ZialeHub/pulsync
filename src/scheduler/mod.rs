use std::collections::HashMap;

use crate::{
    recurrence::Recurrence,
    task::{TaskId, UniqueId},
};

pub mod async_scheduler;
pub mod scheduler;

/// A type alias for the scheduler generic.
///
/// The scheduler is a hash map that stores the tasks scheduled.
/// The key is the unique identifier of the task and the value is the task itself.
/// Tasks can either be synchronous or asynchronous.
pub type Scheduler<T> = HashMap<TaskId, T>;

/// A trait to implement on a scheduler.
///
/// Used to schedule, reschedule, pause, resume, and abort tasks.
pub trait TaskScheduler<T> {
    fn new() -> Self;
    fn schedule(&mut self, task: T, recurrence: Recurrence);
    fn reschedule<U: UniqueId>(&mut self, recurrence: Recurrence);
    fn pause<U: UniqueId>(&mut self);
    fn resume<U: UniqueId>(&mut self);
    fn abort<U: UniqueId>(&mut self);
    fn run(&self, task: T);
}
