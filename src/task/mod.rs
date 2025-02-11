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
///
/// The unique identifier is generated based on the type of the task.
///
/// If a salt is provided, it is also hashed to generate the unique identifier.
///
/// This allow us to generate multiple unique identifiers for the same task type.
pub trait UniqueId
where
    Self: 'static,
{
    fn unique_id(salt: impl ToString) -> TaskId {
        let type_id = TypeId::of::<Self>();
        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);

        hasher.write(salt.to_string().as_bytes());

        hasher.finish()
    }
}

#[cfg(test)]
mod unique_id_tests {
    use super::*;

    #[test]
    fn test_unique_id() {
        struct Task1;
        struct Task2;

        impl UniqueId for Task1 {}
        impl UniqueId for Task2 {}

        assert_ne!(Task1::unique_id(""), Task2::unique_id(""));
        assert_eq!(Task1::unique_id(""), Task1::unique_id(String::new()));
        assert_ne!(Task1::unique_id("0"), Task1::unique_id("1"));
        assert_eq!(Task1::unique_id("0"), Task1::unique_id("0"));
    }
}
