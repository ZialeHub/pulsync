use std::{
    any::{Any, TypeId},
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::task::TaskId;

/// Trait to generate a unique identifier for a type.
///
/// Use the TypeId of the implementing type, hash it and return the result.
///
/// # Example
/// ```rust,ignore
/// struct MyStruct {
///     pub field: String,
/// }
/// impl UniqueId for MyStruct {}
///
/// let id = MyStruct::get_unique_id();
/// ```
pub trait UniqueId: Any {
    fn get_unique_id() -> TaskId {
        let type_id = TypeId::of::<Self>();
        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_unique_id() {
        struct MyStruct;
        impl UniqueId for MyStruct {}

        let mut hasher = DefaultHasher::new();
        TypeId::of::<MyStruct>().hash(&mut hasher);

        let result = MyStruct::get_unique_id();
        let expected = hasher.finish();
        assert_eq!(expected, result);
    }

    #[test]
    fn unicity() {
        struct MyStruct;
        impl UniqueId for MyStruct {}

        let first_id = MyStruct::get_unique_id();
        let second_id = MyStruct::get_unique_id();

        assert!(first_id == second_id);
    }

    #[test]
    fn unicity_different_struct() {
        struct MyFirstStruct;
        impl UniqueId for MyFirstStruct {}

        let first_id = MyFirstStruct::get_unique_id();

        struct MySecondStruct;
        impl UniqueId for MySecondStruct {}

        let second_id = MySecondStruct::get_unique_id();

        assert!(first_id != second_id);
    }
}
