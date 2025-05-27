# Pulsync

A Rust crate for managing synchronous and asynchronous task scheduling with powerful recurrence options.

## Features

- Create and manage a `Scheduler` to handle tasks.
- Support for both **synchronous** and **asynchronous** tasks.
- Flexible `Recurrence` API to control:
  - Timeout between executions.
  - Number of executions.
  - Execution time limits.
- Feature flags:
  - `sync`: Enables synchronous task scheduling.
  - `async`: Enables asynchronous task scheduling.
  - **Note:** You must enable exactly one of the features above; they cannot be used together.
  - `serde`: Enables de/serialize `TaskState<T>` and add a `restart` method to scheduler.

## Installation

Add this crate to your `Cargo.toml`:

```toml,ignore
[dependencies]
pulsync = { version = "0.1", features = ["async"] }
# or
pulsync = { version = "0.1", default-features = false, features = ["sync"] }
```

## Usage

### Synchronous Example

```rust,ignore
use chrono::NaiveDateTime;
use pulsync::prelude::*;

#[derive(Debug, Clone, Task, Salt)]
#[title("MySyncTask for {self.login}")]
#[salt("{self.state}")]
struct MySyncTask {
    login: TaskState<String>,
    state: TaskState<u8>,
}
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

fn main() {
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
}

```

### Asynchronous Example

```rust,ignore
#![cfg(all(feature = "async", not(feature = "sync")))]
use std::{future::Future, pin::Pin};

use pulsync::prelude::*;

#[derive(Debug, Clone, Task, Salt)]
#[title("MyAsyncTask for {self.login}")]
#[salt("{self.state}")]
struct MyAsyncTask {
    login: TaskState<String>,
    state: TaskState<u8>,
}
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

#[tokio::main]
async fn main() {
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
}
```

## License

This project is licensed under the MIT License. See [LICENSE](./LICENSE) for details.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## Links

- **[crates.io](https://crates.io/crates/pulsync)**
- **[Github](https://github.com/ZialeHub/pulsync)**

---

Let me know if you want more details, usage examples, or specific documentation!
