#[cfg(all(feature = "async", not(feature = "sync")))]
use std::{future::Future, pin::Pin};

#[cfg(all(feature = "async", not(feature = "sync")))]
use pulsync::prelude::*;

#[cfg(all(feature = "async", not(feature = "sync")))]
#[derive(Debug, Clone, Task, Salt)]
#[title("MyAsyncTask for {self.login}")]
#[salt("{self.state}")]
struct MyAsyncTask {
    login: TaskState<String>,
    state: TaskState<u8>,
}
#[cfg(all(feature = "async", not(feature = "sync")))]
impl AsyncTaskHandler for MyAsyncTask {
    fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
        Box::pin(async move {
            eprintln!(
                "Number of messages sent to {} = {}",
                self.login.read().unwrap(),
                self.state.read().unwrap()
            );
            *self.state.write().unwrap() += 1;
        })
    }
}

#[cfg(all(feature = "async", not(feature = "sync")))]
#[tokio::main]
async fn main() {
    println!("Async example");
    let mut scheduler = Scheduler::build();
    let task = MyAsyncTask::new(String::from("Pierre"), 0);

    // The task will run every 5 minutes and 30 seconds
    // After 5 times the task will be removed from the scheduler.
    let id = scheduler
        .schedule(
            Box::new(task),
            every(5.minutes()).and(30.seconds()).count(5),
        )
        .unwrap();
    let _ = scheduler.pause(id);
    let _ = scheduler.resume(id);
    let _ = scheduler.abort(id);
    println!("End Async example");
}

#[cfg(all(feature = "sync", not(feature = "async")))]
fn main() {}
