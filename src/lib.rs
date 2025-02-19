#![doc = include_str!("../README.md")]
pub mod prelude;
pub mod recurrence;
pub mod scheduler;
pub mod task;
pub use pulsync_derive::{Salt, Task};

#[doc(inline)]
pub use recurrence::{every, Recurrence, RecurrenceCast};
#[doc(inline)]
pub use scheduler::{Scheduler, TaskScheduler};
#[cfg(feature = "async")]
#[doc(inline)]
pub use task::async_task::{AsyncTask, AsyncTaskHandler};
#[cfg(feature = "sync")]
#[doc(inline)]
pub use task::sync_task::{SyncTask, SyncTaskHandler};
#[doc(inline)]
pub use task::{Salt, Task, TaskId, UniqueId};
