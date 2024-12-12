use std::collections::HashMap;

use crate::{
    recurrence::Recurrence,
    task::{TaskId, UniqueId},
};

pub mod async_scheduler;
pub mod scheduler;

pub type Scheduler<T> = HashMap<TaskId, T>;

pub trait TaskScheduler<T> {
    fn new() -> Self;
    fn schedule(&mut self, task: T, recurrence: Recurrence);
    fn pause<U: UniqueId>(&mut self);
    fn resume<U: UniqueId>(&mut self);
    fn abort<U: UniqueId>(&mut self);
    fn run(&self, task: T);
}
