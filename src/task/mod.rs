use std::hash::{DefaultHasher, Hash, Hasher};

use crate::recurrence::Recurrence;

#[cfg(feature = "async")]
pub mod async_task;
#[cfg(feature = "sync")]
pub mod sync_task;

/// Type alias for the unique identifier of a task.
pub type TaskId = u64;

#[derive(Clone, Copy, PartialEq, Default)]
pub(super) enum TaskStatus {
    Paused,
    #[default]
    Running,
    Abort,
}

/// Trait to represent a task.
/// A task is a unit of work that can be scheduled in the scheduler.
///
/// The task must implement the `UniqueId` trait to generate a unique identifier.
pub trait Task: UniqueId {
    fn title(&self) -> String {
        String::new()
    }
}

/// Trait to generate a unique identifier for a task.
///
/// The unique identifier is generated based on the type of the task.
pub trait UniqueId: Salt
where
    Self: 'static,
{
    fn unique_id(&self, recurrence: Recurrence) -> TaskId {
        let mut hasher = DefaultHasher::new();
        recurrence.hash(&mut hasher);
        let salt = self.salt();
        salt.hash(&mut hasher);
        hasher.finish()
    }
}

/// Trait to generate a salt for a task.
///
/// The salt is used to generate a unique identifier for each instance of a task.
pub trait Salt {
    fn salt(&self) -> String {
        String::new()
    }
}
