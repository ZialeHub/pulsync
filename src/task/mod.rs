use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, RwLock},
};

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

#[derive(Debug, Clone)]
pub struct TaskState<T>(Arc<RwLock<T>>);
impl<T> TaskState<T> {
    pub fn new(data: T) -> Self {
        Self(Arc::new(RwLock::new(data)))
    }
}

impl<T> std::ops::Deref for TaskState<T> {
    type Target = Arc<RwLock<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for TaskState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "serde")]
impl<T: Clone + serde::Serialize> serde::Serialize for TaskState<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (*self.read().unwrap()).clone().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Clone + serde::Deserialize<'de>> serde::Deserialize<'de> for TaskState<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = T::deserialize(deserializer)?;
        Ok(TaskState(Arc::new(RwLock::new(data))))
    }
}
