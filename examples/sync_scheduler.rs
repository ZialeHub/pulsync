#[cfg(all(feature = "sync", not(feature = "async")))]
use chrono::NaiveDateTime;
#[cfg(all(feature = "sync", not(feature = "async")))]
use pulsync::prelude::*;

#[cfg(all(feature = "sync", not(feature = "async")))]
#[derive(Debug, Clone, Task, Salt)]
#[title("MySyncTask for {self.login}")]
#[salt("{self.state}")]
struct MySyncTask {
    login: TaskState<String>,
    state: TaskState<u8>,
}
#[cfg(all(feature = "sync", not(feature = "async")))]
impl SyncTaskHandler for MySyncTask {
    fn run(&self) {
        eprintln!(
            "Number of messages sent to {} = {}",
            self.login.read().unwrap(),
            self.state.read().unwrap()
        );
        *self.state.write().unwrap() += 1;
    }
}

#[cfg(all(feature = "sync", not(feature = "async")))]
fn main() {
    println!("Sync example");
    let mut scheduler = Scheduler::build();
    let task = MySyncTask::new(String::from("Pierre"), 0);

    // The task will run every 3 days until 2026-01-01 00:00:00
    let _id = scheduler
        .schedule(
            Box::new(task),
            every(3.days()).until_datetime(
                NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            ),
        )
        .unwrap();

    // titles = ["[SyncTask({id})] MySyncTask for Pierre started at {task start datetime}"]
    let _titles = scheduler.get();
    println!("End Sync example");
}

#[cfg(all(feature = "async", not(feature = "sync")))]
fn main() {}
