#[cfg(feature = "sync")]
#[cfg(test)]
mod test {
    use std::{
        sync::{Arc, RwLock},
        time::Duration,
    };

    use pulsync::prelude::*;
    use pulsync_derive::{Salt, Task};

    #[test]
    fn sync_scheduler_single_task() -> Result<(), ()> {
        // Sync Task
        let mut scheduler = Scheduler::build();
        #[derive(Clone, Task, Salt)]
        #[title("MySyncTask state=|{self.state}|")]
        #[salt("MySyncTaskSALT")]
        struct MySyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MySyncTask::new(0);
        impl SyncTaskHandler for MySyncTask {
            fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }
        }

        // Run the task once
        scheduler.run(task.clone());
        std::thread::sleep(Duration::from_secs(1));
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 1);
        // Run the task every second
        let id = scheduler
            .schedule(Box::new(task.clone()), every(1.seconds()))
            .unwrap();
        std::thread::sleep(Duration::from_secs(5));
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 6);
        // Run the task every 3 seconds, 3 times
        let id = scheduler
            .schedule(Box::new(task.clone()), every(3.seconds()).count(3))
            .unwrap();
        std::thread::sleep(Duration::from_secs(10));
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 9);
        // Run the task every 1 minute and 2 seconds
        let id = scheduler
            .schedule(
                Box::new(task.clone()),
                every(1.minutes()).and(2.seconds()).count(1),
            )
            .unwrap();
        std::thread::sleep(Duration::from_secs(120));
        scheduler.pause(id);
        scheduler.resume(id);
        scheduler.abort(id);
        scheduler.pause(id);
        scheduler.resume(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 10);
        Ok(())
    }

    #[test]
    fn sync_scheduler_run_before_and_after() -> Result<(), ()> {
        // Sync Task
        let mut scheduler = Scheduler::build();
        #[derive(Clone, Task, Salt)]
        #[title("MySyncTask state=|{self.state}|")]
        #[salt("MySyncTaskSALT")]
        struct MySyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MySyncTask::new(0);
        impl SyncTaskHandler for MySyncTask {
            fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }
        }

        // Run the task every 3 seconds after timer
        let id = scheduler
            .schedule(Box::new(task.clone()), every(3.seconds()).run_after(true))
            .unwrap();
        std::thread::sleep(Duration::from_secs(9));
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 2);

        // Run the task every 3 seconds before timer
        let id = scheduler
            .schedule(Box::new(task.clone()), every(3.seconds()))
            .unwrap();
        std::thread::sleep(Duration::from_secs(9));
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn sync_scheduler_multiple_instance_of_task() -> Result<(), ()> {
        let mut scheduler = Scheduler::build();

        #[derive(Debug, Clone, Task, Salt)]
        #[salt("{self.login}")]
        struct MySyncTask {
            login: Arc<RwLock<String>>,
            state: Arc<RwLock<u8>>,
        }
        impl SyncTaskHandler for MySyncTask {
            fn run(&self) {
                let mut state = self.state.write().unwrap();
                eprintln!(
                    "Message from {} with value {}",
                    self.login.read().unwrap(),
                    *state
                );
                *state += 1;
            }
        }

        let state = Arc::new(RwLock::new(0));
        let task = MySyncTask {
            login: Arc::new(RwLock::new(String::from("pierre"))),
            state: state.clone(),
        };
        let task2 = MySyncTask {
            login: Arc::new(RwLock::new(String::from("jean"))),
            state: state.clone(),
        };
        let task3 = MySyncTask {
            login: Arc::new(RwLock::new(String::from("lucie"))),
            state: state.clone(),
        };
        // Run the task once
        scheduler.run(task.clone());
        std::thread::sleep(Duration::from_secs(1));
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 1);
        // Run the tasks every 5 seconds
        let id1 = scheduler
            .schedule(Box::new(task.clone()), every(5.seconds()))
            .unwrap();
        let id2 = scheduler
            .schedule(Box::new(task2.clone()), every(5.seconds()))
            .unwrap();
        let id3 = scheduler
            .schedule(Box::new(task3.clone()), every(5.seconds()))
            .unwrap();
        std::thread::sleep(Duration::from_secs(15));
        scheduler.abort(id1);
        scheduler.abort(id2);
        scheduler.abort(id3);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 10);
        Ok(())
    }

    #[test]
    fn sync_reschedule_task() -> Result<(), ()> {
        // Sync Task
        let mut scheduler = Scheduler::build();
        #[derive(Clone, Task, Salt)]
        #[salt("MySyncTaskSALT")]
        struct MySyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MySyncTask::new(0);
        impl SyncTaskHandler for MySyncTask {
            fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }
        }

        // Run the task once
        let id = scheduler
            .schedule(Box::new(task.clone()), every(1.seconds()))
            .unwrap();
        std::thread::sleep(Duration::from_secs(3));
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 3);
        let _new_id = scheduler.reschedule(id, every(5.seconds())).unwrap();
        std::thread::sleep(Duration::from_secs(10));
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn get_task_list() {
        let mut scheduler = Scheduler::build();
        #[derive(Debug, Clone, Task, Salt)]
        #[title("First Task")]
        #[salt("MySyncTask: {self.state}")]
        struct MySyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MySyncTask::new(0);
        impl SyncTaskHandler for MySyncTask {
            fn run(&self) {
                *self.state.write().unwrap() += 1;
            }
        }
        #[derive(Debug, Clone, Task, Salt)]
        #[title("Second Task with login {self.login}")]
        #[salt("MySyncTask2: {self.login}")]
        struct MySyncTask2 {
            login: Arc<RwLock<String>>,
            age: Arc<RwLock<u32>>,
        }
        let task2 = MySyncTask2::new(String::from("Pierre"), 25);
        impl SyncTaskHandler for MySyncTask2 {
            fn run(&self) {
                eprintln!(
                    "Message from {} with value {}",
                    self.login.read().unwrap(),
                    self.age.read().unwrap()
                );
            }
        }
        let id = scheduler
            .schedule(Box::new(task), every(1.seconds()).count(3))
            .unwrap();
        let id2 = scheduler
            .schedule(Box::new(task2), every(1.seconds()).count(3))
            .unwrap();
        let task_list = scheduler.get();
        assert_eq!(task_list.len(), 2);
        assert!(task_list
            .iter()
            .find(|task| task.starts_with(&format!("[SyncTask({id})] First Task started at ")))
            .is_some());
        assert!(task_list
            .iter()
            .find(|task| task.starts_with(&format!(
                "[SyncTask({id2})] Second Task with login Pierre started at "
            )))
            .is_some());
    }

    #[test]
    fn async_run_until() -> Result<(), ()> {
        let mut scheduler = Scheduler::build();
        #[derive(Debug, Clone, Task, Salt)]
        #[title("MySyncTask for {self.login}")]
        #[salt("MySyncTask: {self.state}")]
        struct MySyncTask {
            login: Arc<RwLock<String>>,
            state: Arc<RwLock<u8>>,
        }
        let value1 = Arc::new(RwLock::new(0));
        let login1 = Arc::new(RwLock::new(String::from("Jean")));
        let value2 = Arc::new(RwLock::new(0));
        let login2 = Arc::new(RwLock::new(String::from("Pierre")));
        let task1 = MySyncTask {
            login: login1,
            state: value1.clone(),
        };
        let task2 = MySyncTask {
            login: login2,
            state: value2.clone(),
        };
        impl SyncTaskHandler for MySyncTask {
            fn run(&self) {
                *self.state.write().unwrap() += 1;
            }
        }
        let _id1 = scheduler
            .schedule(Box::new(task1), every(2.seconds()).until(10.seconds()))
            .unwrap();
        let _id2 = scheduler
            .schedule(Box::new(task2), every(3.seconds()).until(3.seconds()))
            .unwrap();
        std::thread::sleep(Duration::from_secs(15));
        assert_eq!(scheduler.get().len(), 0);
        assert_eq!(*value1.read().unwrap(), 5);
        assert_eq!(*value2.read().unwrap(), 1);
        Ok(())
    }

    #[test]
    fn sync_run_until_datetime() -> Result<(), ()> {
        let mut scheduler = Scheduler::build();
        #[derive(Debug, Clone, Task, Salt)]
        #[title("MySyncTask for {self.login}")]
        #[salt("MySyncTask: {self.state}")]
        struct MySyncTask {
            login: Arc<RwLock<String>>,
            state: Arc<RwLock<u8>>,
        }
        let value1 = Arc::new(RwLock::new(0));
        let login1 = Arc::new(RwLock::new(String::from("Jean")));
        let value2 = Arc::new(RwLock::new(0));
        let login2 = Arc::new(RwLock::new(String::from("Pierre")));
        let task1 = MySyncTask {
            login: login1,
            state: value1.clone(),
        };
        let task2 = MySyncTask {
            login: login2,
            state: value2.clone(),
        };
        impl SyncTaskHandler for MySyncTask {
            fn run(&self) {
                *self.state.write().unwrap() += 1;
            }
        }
        let mut ten_seconds_in_future = chrono::Utc::now().naive_utc();
        ten_seconds_in_future += chrono::Duration::seconds(10);
        let _id1 = scheduler
            .schedule(
                Box::new(task1),
                every(2.seconds()).until_datetime(ten_seconds_in_future),
            )
            .unwrap();
        let mut three_seconds_in_future = chrono::Utc::now().naive_utc();
        three_seconds_in_future += chrono::Duration::seconds(3);
        let _id2 = scheduler
            .schedule(
                Box::new(task2),
                every(3.seconds()).until_datetime(three_seconds_in_future),
            )
            .unwrap();
        std::thread::sleep(Duration::from_secs(15));
        assert_eq!(scheduler.get().len(), 0);
        assert_eq!(*value1.read().unwrap(), 5);
        assert_eq!(*value2.read().unwrap(), 1);
        Ok(())
    }
}

#[cfg(feature = "async")]
#[cfg(test)]
mod test {
    use std::{
        future::Future,
        pin::Pin,
        sync::{Arc, RwLock},
        time::Duration,
    };

    use pulsync::prelude::*;
    use pulsync_derive::{Salt, Task};

    #[tokio::test]
    async fn async_scheduler_single_task() -> Result<(), ()> {
        // Async Task
        let mut scheduler = Scheduler::build();
        #[derive(Clone, Task, Salt)]
        #[title("MyAsyncTask state=|{self.state}|")]
        #[salt("MyAsyncTaskSALT")]
        struct MyAsyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MyAsyncTask::new(0);
        impl AsyncTaskHandler for MyAsyncTask {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    let mut state = self.state.write().unwrap();
                    *state += 1;
                })
            }
        }

        // Run the task once
        scheduler.run(task.clone());
        tokio::time::sleep(Duration::from_secs(1)).await;
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 1);
        // Run the task every second
        let id = scheduler
            .schedule(Box::new(task.clone()), every(1.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 6);
        // Run the task every 3 seconds, 3 times
        let id = scheduler
            .schedule(Box::new(task.clone()), every(3.seconds()).count(3))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 9);
        // Run the task every 1 minute and 2 seconds
        let id = scheduler
            .schedule(
                Box::new(task.clone()),
                every(1.minutes()).and(2.seconds()).count(1),
            )
            .unwrap();
        tokio::time::sleep(Duration::from_secs(120)).await;
        scheduler.pause(id);
        scheduler.resume(id);
        scheduler.abort(id);
        scheduler.pause(id);
        scheduler.resume(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 10);
        Ok(())
    }

    #[tokio::test]
    async fn async_scheduler_run_before_and_after() -> Result<(), ()> {
        // Async Task
        let mut scheduler = Scheduler::build();
        #[derive(Clone, Task, Salt)]
        #[title("MyAsyncTask state=|{self.state}|")]
        #[salt("MyAsyncTaskSALT")]
        struct MyAsyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MyAsyncTask::new(0);
        impl AsyncTaskHandler for MyAsyncTask {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    let mut state = self.state.write().unwrap();
                    *state += 1;
                })
            }
        }

        // Run the task every 3 seconds after timer
        let id = scheduler
            .schedule(Box::new(task.clone()), every(3.seconds()).run_after(true))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(9)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 2);

        // Run the task every 3 seconds before timer
        let id = scheduler
            .schedule(Box::new(task.clone()), every(3.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(9)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 5);
        Ok(())
    }

    #[tokio::test]
    async fn async_reschedule_task() -> Result<(), ()> {
        // Async Task
        let mut scheduler = Scheduler::build();
        #[derive(Clone, Task, Salt)]
        #[salt("MyAsyncTaskSALT")]
        struct MyAsyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MyAsyncTask::new(0);
        impl AsyncTaskHandler for MyAsyncTask {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    let mut state = self.state.write().unwrap();
                    *state += 1;
                })
            }
        }

        // Run the task once
        let id = scheduler
            .schedule(Box::new(task.clone()), every(1.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(3)).await;
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 3);
        let _new_id = scheduler.reschedule(id, every(5.seconds())).unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *task.state.read().unwrap());
        assert_eq!(*task.state.read().unwrap(), 5);
        Ok(())
    }

    #[tokio::test]
    async fn async_scheduler_multiple_instance_of_task() -> Result<(), ()> {
        let mut scheduler = Scheduler::build();

        #[derive(Debug, Clone, Task, Salt)]
        #[salt("{self.login}")]
        struct MyAsyncTask {
            login: Arc<RwLock<String>>,
            state: Arc<RwLock<u8>>,
        }
        impl AsyncTaskHandler for MyAsyncTask {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    let mut state = self.state.write().unwrap();
                    eprintln!(
                        "Message from {} with value {}",
                        self.login.read().unwrap(),
                        *state
                    );
                    *state += 1;
                })
            }
        }

        let state = Arc::new(RwLock::new(0));
        let task = MyAsyncTask {
            login: Arc::new(RwLock::new(String::from("pierre"))),
            state: state.clone(),
        };
        let task2 = MyAsyncTask {
            login: Arc::new(RwLock::new(String::from("jean"))),
            state: state.clone(),
        };
        let task3 = MyAsyncTask {
            login: Arc::new(RwLock::new(String::from("lucie"))),
            state: state.clone(),
        };
        // Run the task once
        scheduler.run(task.clone());
        tokio::time::sleep(Duration::from_secs(1)).await;
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 1);
        // Run the tasks every 5 seconds
        let id1 = scheduler
            .schedule(Box::new(task.clone()), every(5.seconds()))
            .unwrap();
        let id2 = scheduler
            .schedule(Box::new(task2.clone()), every(5.seconds()))
            .unwrap();
        let id3 = scheduler
            .schedule(Box::new(task3.clone()), every(5.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(15)).await;
        scheduler.abort(id1);
        scheduler.abort(id2);
        scheduler.abort(id3);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 10);
        Ok(())
    }

    #[tokio::test]
    async fn get_task_list() {
        let mut scheduler = Scheduler::build();
        #[derive(Debug, Clone, Task, Salt)]
        #[title("First Task")]
        #[salt("MyAsyncTask: {self.state}")]
        struct MyAsyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MyAsyncTask::new(0);
        impl AsyncTaskHandler for MyAsyncTask {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    *self.state.write().unwrap() += 1;
                })
            }
        }
        #[derive(Debug, Clone, Task, Salt)]
        #[title("Second Task with login {self.login}")]
        #[salt("MyAsyncTask2: {self.login}")]
        struct MyAsyncTask2 {
            login: Arc<RwLock<String>>,
            age: Arc<RwLock<u32>>,
        }
        let task2 = MyAsyncTask2::new(String::from("Pierre"), 25);
        impl AsyncTaskHandler for MyAsyncTask2 {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    eprintln!(
                        "Message from {} with value {}",
                        self.login.read().unwrap(),
                        self.age.read().unwrap()
                    );
                })
            }
        }
        let id = scheduler
            .schedule(Box::new(task), every(1.seconds()).count(3))
            .unwrap();
        let id2 = scheduler
            .schedule(Box::new(task2), every(1.seconds()).count(3))
            .unwrap();
        let task_list = scheduler.get();
        assert_eq!(task_list.len(), 2);
        assert!(task_list
            .iter()
            .find(|task| task.starts_with(&format!("[AsyncTask({id})] First Task started at ")))
            .is_some());
        assert!(task_list
            .iter()
            .find(|task| task.starts_with(&format!(
                "[AsyncTask({id2})] Second Task with login Pierre started at "
            )))
            .is_some());
    }

    #[tokio::test]
    async fn async_run_until() -> Result<(), ()> {
        let mut scheduler = Scheduler::build();
        #[derive(Debug, Clone, Task, Salt)]
        #[title("MyAsyncTask for {self.login}")]
        #[salt("MyAsyncTask: {self.state}")]
        struct MyAsyncTask {
            login: Arc<RwLock<String>>,
            state: Arc<RwLock<u8>>,
        }
        let value1 = Arc::new(RwLock::new(0));
        let login1 = Arc::new(RwLock::new(String::from("Jean")));
        let value2 = Arc::new(RwLock::new(0));
        let login2 = Arc::new(RwLock::new(String::from("Pierre")));
        let task1 = MyAsyncTask {
            login: login1,
            state: value1.clone(),
        };
        let task2 = MyAsyncTask {
            login: login2,
            state: value2.clone(),
        };
        impl AsyncTaskHandler for MyAsyncTask {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    *self.state.write().unwrap() += 1;
                })
            }
        }
        let _id1 = scheduler
            .schedule(Box::new(task1), every(2.seconds()).until(10.seconds()))
            .unwrap();
        let _id2 = scheduler
            .schedule(Box::new(task2), every(3.seconds()).until(3.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(15)).await;
        assert_eq!(scheduler.get().len(), 0);
        assert_eq!(*value1.read().unwrap(), 5);
        assert_eq!(*value2.read().unwrap(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn async_run_until_datetime() -> Result<(), ()> {
        let mut scheduler = Scheduler::build();
        #[derive(Debug, Clone, Task, Salt)]
        #[title("MyAsyncTask for {self.login}")]
        #[salt("MyAsyncTask: {self.state}")]
        struct MyAsyncTask {
            login: Arc<RwLock<String>>,
            state: Arc<RwLock<u8>>,
        }
        let value1 = Arc::new(RwLock::new(0));
        let login1 = Arc::new(RwLock::new(String::from("Jean")));
        let value2 = Arc::new(RwLock::new(0));
        let login2 = Arc::new(RwLock::new(String::from("Pierre")));
        let task1 = MyAsyncTask {
            login: login1,
            state: value1.clone(),
        };
        let task2 = MyAsyncTask {
            login: login2,
            state: value2.clone(),
        };
        impl AsyncTaskHandler for MyAsyncTask {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    *self.state.write().unwrap() += 1;
                })
            }
        }
        let mut ten_seconds_in_future = chrono::Utc::now().naive_utc();
        ten_seconds_in_future += chrono::Duration::seconds(10);
        let _id1 = scheduler
            .schedule(
                Box::new(task1),
                every(2.seconds()).until_datetime(ten_seconds_in_future),
            )
            .unwrap();
        let mut three_seconds_in_future = chrono::Utc::now().naive_utc();
        three_seconds_in_future += chrono::Duration::seconds(3);
        let _id2 = scheduler
            .schedule(
                Box::new(task2),
                every(3.seconds()).until_datetime(three_seconds_in_future),
            )
            .unwrap();
        tokio::time::sleep(Duration::from_secs(15)).await;
        assert_eq!(scheduler.get().len(), 0);
        assert_eq!(*value1.read().unwrap(), 5);
        assert_eq!(*value2.read().unwrap(), 1);
        Ok(())
    }
}
