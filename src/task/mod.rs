use std::{
    any::TypeId,
    hash::{DefaultHasher, Hash, Hasher},
};

pub mod async_task;
pub mod task;

pub type TaskId = u64;

pub trait Task: UniqueId {}

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
