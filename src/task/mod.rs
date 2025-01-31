use std::{
    any::TypeId,
    hash::{DefaultHasher, Hash, Hasher},
};

pub mod async_task;
pub mod task;

/// Type alias for the unique identifier of a task.
pub type TaskId = u64;

/// Trait to represent a task.
/// A task is a unit of work that can be scheduled in the scheduler.\
/// The task must implement the `UniqueId` trait to generate a unique identifier.
///
///
pub trait Task: UniqueId {}

/// Trait to generate a unique identifier for a task.
/// The unique identifier is generated based on the type of the task.
pub trait UniqueId
where
    Self: 'static,
{
    fn unique_id() -> TaskId {
        let type_id = TypeId::of::<Self>();
        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);
        hasher.finish()
    }
}
