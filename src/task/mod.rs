use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, RwLock},
};

use chrono::NaiveDateTime;

use crate::recurrence::Recurrence;

#[cfg(feature = "async")]
pub mod async_task;
#[cfg(feature = "sync")]
pub mod sync_task;

/// Type alias for the unique identifier of a task.
pub type TaskId = u64;

/// Type alias for the label returned from scheduled tasks.
pub type TaskLabel = String;

/// Struct used to return scheduled tasks details from a scheduler
#[derive(Debug, Clone)]
pub struct TaskDetails {
    pub id: TaskId,
    pub label: TaskLabel,
    pub recurrence: Arc<RwLock<Recurrence>>,
    pub status: TaskStatus,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TaskStatus {
    Paused,
    #[default]
    Running,
    Abort,
}
impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Paused => "inactive",
                Self::Abort => "canceled",
                Self::Running => "active",
            }
        )
    }
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
