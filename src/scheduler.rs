use std::{
    any::TypeId,
    collections::HashMap,
    future::Future,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};

pub type TaskId = u64;

pub type Scheduler = HashMap<TaskId, SyncTask>;
pub type AsyncScheduler = HashMap<TaskId, AsyncTask>;

pub trait TaskScheduler<T: Task> {
    fn new() -> Self;
    fn schedule(&mut self, task: T, recurrence: Recurrence);
    fn pause<U: Task>(&mut self);
    fn resume<U: Task>(&mut self);
    fn abort<U: Task>(&mut self);
    fn run(&self, task: T);
}

impl<T> TaskScheduler<T> for AsyncScheduler
where
    T: Task + AsyncTaskHandler + Send + Sync + 'static,
{
    fn new() -> Self {
        HashMap::new()
    }

    fn schedule(&mut self, task: T, recurrence: Recurrence) {
        let id = T::unique_id();
        if self.contains_key(&id) {
            eprintln!("Task ({id}): already exists");
            tracing::info!("Task ({id}): already exists");
            return;
        }
        let state = Arc::new(AtomicBool::new(true));
        let recurrence = Arc::new(RwLock::new(recurrence));
        let handler = {
            let state = state.clone();
            let recurrence = recurrence.clone();
            tokio::spawn(async move {
                loop {
                    {
                        let mut recurrence = recurrence.write().unwrap();
                        match recurrence.count {
                            None => {}
                            Some(count) if count == 0 => break,
                            Some(count) => {
                                recurrence.count = Some(count - 1);
                            }
                        }
                    }
                    if state.load(Ordering::Relaxed) {
                        task.run().await;
                    }
                    {
                        let duration = {
                            let read_guard = recurrence.read().unwrap();
                            read_guard.unit
                        };
                        let _ = tokio::time::sleep(duration.into()).await;
                    }
                }
            })
        };
        let task = AsyncTask {
            id,
            state,
            handler,
            recurrence,
        };
        self.insert(task.id, task);
    }

    fn pause<U: Task>(&mut self) {
        // Pause the task
        let id = U::unique_id();
        let Some(task) = self.get_mut(&U::unique_id()) else {
            eprintln!("Task ({id}): not found");
            tracing::info!("Task ({id}): not found");
            return;
        };
        if !task.state.load(Ordering::Relaxed) {
            eprintln!("Task ({id}): already paused");
            tracing::info!("Task ({id}): already paused");
            return;
        }
        task.state.store(false, Ordering::Relaxed);
    }

    fn resume<U: Task>(&mut self) {
        // Resume the task
        let id = U::unique_id();
        let Some(task) = self.get_mut(&T::unique_id()) else {
            eprintln!("Task ({id}): not found");
            tracing::info!("Task ({id}): not found");
            return;
        };
        if task.state.load(Ordering::Relaxed) {
            eprintln!("Task ({id}): already resumed");
            tracing::info!("Task ({id}): already resumed");
            return;
        }
        task.state.store(true, Ordering::Relaxed);
    }

    fn abort<U: Task>(&mut self) {
        // Abort the task
        let id = U::unique_id();
        let Some(task) = self.remove(&id) else {
            eprintln!("Task ({id}): not found");
            tracing::info!("Task ({id}): not found");
            return;
        };
        task.handler.abort();
    }

    fn run(&self, task: T) {
        // Run task once
        tokio::spawn(async move { task.run().await });
    }
}
impl<T> TaskScheduler<T> for Scheduler
where
    T: Task + SyncTaskHandler + Send + Sync + 'static,
{
    fn new() -> Self {
        HashMap::new()
    }

    fn schedule(&mut self, task: T, recurrence: Recurrence) {
        let id = T::unique_id();
        if self.contains_key(&id) {
            eprintln!("Task ({id}): already exists");
            tracing::info!("Task ({id}): already exists");
            return;
        }
        let state = Arc::new(AtomicBool::new(true));
        let recurrence = Arc::new(RwLock::new(recurrence));
        let handler = {
            let state = state.clone();
            let recurrence = recurrence.clone();
            tokio::spawn(async move {
                loop {
                    {
                        let mut recurrence = recurrence.write().unwrap();
                        match recurrence.count {
                            None => {}
                            Some(count) if count == 0 => break,
                            Some(count) => {
                                recurrence.count = Some(count - 1);
                            }
                        }
                    }
                    if state.load(Ordering::Relaxed) {
                        task.run();
                    }
                    {
                        let duration = {
                            let read_guard = recurrence.read().unwrap();
                            read_guard.unit
                        };
                        let _ = tokio::time::sleep(duration.into()).await;
                    }
                }
            })
        };
        let task = SyncTask {
            id,
            state,
            handler,
            recurrence,
        };
        self.insert(task.id, task);
    }

    fn pause<U: Task>(&mut self) {
        // Pause the task
        let id = U::unique_id();
        let Some(task) = self.get_mut(&id) else {
            eprintln!("Task ({id}): not found");
            tracing::info!("Task ({id}): not found");
            return;
        };
        if !task.state.load(Ordering::Relaxed) {
            eprintln!("Task ({id}): already paused");
            tracing::info!("Task ({id}): already paused");
            return;
        }
        task.state.store(false, Ordering::Relaxed);
    }

    fn resume<U: Task>(&mut self) {
        // Resume the task
        let id = U::unique_id();
        let Some(task) = self.get_mut(&id) else {
            eprintln!("Task ({id}): not found");
            tracing::info!("Task ({id}): not found");
            return;
        };
        if task.state.load(Ordering::Relaxed) {
            eprintln!("Task ({id}): already resumed");
            tracing::info!("Task ({id}): already resumed");
            return;
        }
        task.state.store(true, Ordering::Relaxed);
    }

    fn abort<U: Task>(&mut self) {
        // Abort the task
        let id = U::unique_id();
        let Some(task) = self.remove(&id) else {
            eprintln!("Task ({id}): not found");
            tracing::info!("Task ({id}): not found");
            return;
        };
        task.handler.abort();
    }

    fn run(&self, task: T) {
        // Run task once
        tokio::spawn(async move { task.run() });
    }
}

pub trait Task: UniqueId {}
pub trait AsyncTaskHandler: Task {
    fn run(&self) -> impl Future<Output = ()> + Send;
}
pub trait SyncTaskHandler: Task {
    fn run(&self);
}

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

pub struct AsyncTask {
    pub id: TaskId,
    pub state: Arc<AtomicBool>,
    pub handler: tokio::task::JoinHandle<()>,
    pub recurrence: Arc<RwLock<Recurrence>>,
}
impl std::fmt::Debug for AsyncTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncTask")
            .field("id", &self.id)
            .field("state", &self.state)
            .finish()
    }
}

pub struct SyncTask {
    pub id: TaskId,
    pub state: Arc<AtomicBool>,
    pub handler: tokio::task::JoinHandle<()>,
    pub recurrence: Arc<RwLock<Recurrence>>,
}
impl std::fmt::Debug for SyncTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncTask")
            .field("id", &self.id)
            .field("state", &self.state)
            .finish()
    }
}

pub type AsyncFunc = dyn Future<Output = ()> + Send + 'static;

pub type SyncFunc = dyn FnMut() -> () + Send;

#[derive(Debug, Clone)]
pub struct Recurrence {
    pub unit: RecurrenceUnit,
    pub count: Option<u64>,
}
impl Recurrence {
    pub fn and(mut self, unit: RecurrenceUnit) -> Self {
        *self.unit = *self.unit + *unit;
        self
    }

    pub fn count(mut self, count: u64) -> Self {
        self.count = Some(count);
        self
    }
}
pub fn every(unit: RecurrenceUnit) -> Recurrence {
    Recurrence { unit, count: None }
}

/// Recurrence in seconds
#[derive(Debug, Clone, Copy)]
pub struct RecurrenceUnit(u64);
impl std::ops::Deref for RecurrenceUnit {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RecurrenceUnit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Into<Duration> for RecurrenceUnit {
    fn into(self) -> Duration {
        Duration::from_secs(self.0)
    }
}

pub trait RecurrenceCast
where
    Self: Into<u64>,
{
    fn seconds(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into())
    }

    fn minutes(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60)
    }

    fn hours(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60)
    }

    fn days(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60 * 24)
    }

    fn weeks(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60 * 24 * 7)
    }

    fn months(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60 * 24 * 30)
    }

    fn years(self) -> RecurrenceUnit {
        RecurrenceUnit(self.into() * 60 * 60 * 24 * 365)
    }
}

//impl RecurrenceCast for u8 {}
//impl RecurrenceCast for u16 {}
//impl RecurrenceCast for u32 {}
impl RecurrenceCast for u64 {}

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use super::*;

    #[tokio::test]
    async fn compile_test() -> Result<(), ()> {
        // Sync task
        let mut scheduler = Scheduler::new();
        #[derive(Clone)]
        struct MySyncTask {
            state: Arc<RwLock<u8>>,
        }
        let state = Arc::new(RwLock::new(0));
        let task = MySyncTask {
            state: state.clone(),
        };
        impl UniqueId for MySyncTask {}
        impl Task for MySyncTask {}
        impl SyncTaskHandler for MySyncTask {
            fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }
        }
        // Run the task once
        scheduler.run(task.clone());
        // Run the task every second
        scheduler.schedule(task.clone(), every(1.seconds()));
        // Run the task every 3 seconds, 3 times
        scheduler.schedule(task.clone(), every(3.seconds()).count(3));
        // Run the task every 1 minute and 2 seconds
        scheduler.schedule(task, every(1.minutes()).and(2.seconds()));
        //scheduler.pause(MySyncTask::unique_id());
        //scheduler.resume(MySyncTask::unique_id());
        //scheduler.abort(MySyncTask::unique_id());

        //<HashMap<u64, SyncTask> as TaskScheduler<MySyncTask>>::pause(
        //    &mut scheduler,
        //    MySyncTask::unique_id(),
        //);
        //<HashMap<u64, SyncTask> as TaskScheduler<MySyncTask>>::resume(
        //    &mut scheduler,
        //    MySyncTask::unique_id(),
        //);
        //<HashMap<u64, SyncTask> as TaskScheduler<MySyncTask>>::abort(
        //    &mut scheduler,
        //    MySyncTask::unique_id(),
        //);
        eprintln!("SYNC scheduler = {:?}", scheduler);
        tokio::time::sleep(Duration::from_secs(3)).await;
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 4);

        // Async Task
        let mut scheduler = AsyncScheduler::new();
        #[derive(Clone)]
        struct MyAsyncTask {
            state: Arc<RwLock<u8>>,
        }
        let state = Arc::new(RwLock::new(0));
        let task = MyAsyncTask {
            state: state.clone(),
        };
        impl UniqueId for MyAsyncTask {}
        impl Task for MyAsyncTask {}
        impl AsyncTaskHandler for MyAsyncTask {
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }
        }

        // Run the task once
        scheduler.run(task.clone());
        tokio::time::sleep(Duration::from_secs(1)).await;
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 1);
        // Run the task every second
        scheduler.schedule(task.clone(), every(1.seconds()));
        tokio::time::sleep(Duration::from_secs(5)).await;
        //<Scheduler<AsyncTask> as TaskScheduler<MyAsyncTask>>::abort(
        //    &mut scheduler,
        //    MyAsyncTask::unique_id(),
        //);
        scheduler.abort::<MyAsyncTask>();
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 6);
        // Run the task every 3 seconds, 3 times
        scheduler.schedule(task.clone(), every(3.seconds()).count(3));
        tokio::time::sleep(Duration::from_secs(10)).await;
        //scheduler.abort(MyAsyncTask::unique_id());
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 9);
        // Run the task every 1 minute and 2 seconds
        scheduler.schedule(task, every(1.minutes()).and(2.seconds()).count(1));
        tokio::time::sleep(Duration::from_secs(120)).await;
        //scheduler.abort(MyAsyncTask::unique_id());
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 10);
        //scheduler.pause(MyAsyncTask::unique_id());
        //scheduler.resume(MyAsyncTask::unique_id());
        //scheduler.abort(MyAsyncTask::unique_id());

        //<HashMap<u64, AsyncTask> as TaskScheduler<MyAsyncTask>>::pause(
        //    &mut scheduler,
        //    MySyncTask::unique_id(),
        //);
        //<HashMap<u64, AsyncTask> as TaskScheduler<MyAsyncTask>>::resume(
        //    &mut scheduler,
        //    MySyncTask::unique_id(),
        //);
        //<HashMap<u64, AsyncTask> as TaskScheduler<MyAsyncTask>>::abort(
        //    &mut scheduler,
        //    MyAsyncTask::unique_id(),
        //);
        assert!(false);
        Ok(())
    }
}
