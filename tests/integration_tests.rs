#[cfg(test)]
mod test {
    use std::{
        future::Future,
        pin::Pin,
        sync::{Arc, RwLock},
        time::Duration,
    };

    use pulsync::{prelude::*, task::Salt};
    use pulsync_derive::{Salt, Task};

    //#[tokio::test]
    //async fn sync_scheduler_single_task() -> Result<(), ()> {
    //    // Sync task
    //    let mut scheduler: Arc<RwLock<HashMap<TaskId, SyncTask<_>>>> = Scheduler::build();
    //    #[derive(Debug, Clone)]
    //    struct MySyncTask {
    //        state: Arc<RwLock<u8>>,
    //    }
    //    let state = Arc::new(RwLock::new(0));
    //    let task = MySyncTask {
    //        state: state.clone(),
    //    };
    //    impl Salt for MySyncTask {
    //        fn salt(&self) -> String {
    //            "MySyncTask".to_string()
    //        }
    //    }
    //    impl UniqueId for MySyncTask {}
    //    impl Task for MySyncTask {}
    //    impl SyncTaskHandler for MySyncTask {
    //        fn run(&self) {
    //            let mut state = self.state.write().unwrap();
    //            *state += 1;
    //        }
    //    }
    //    // Run the task once
    //    scheduler.run(task.clone());
    //    tokio::time::sleep(Duration::from_secs(1)).await;
    //    eprintln!("State = {:?}", *state.read().unwrap());
    //    assert_eq!(*state.read().unwrap(), 1);
    //    // Run the task every second
    //    let id = scheduler
    //        .schedule(task.clone(), every(1.seconds()))
    //        .unwrap();
    //    tokio::time::sleep(Duration::from_secs(5)).await;
    //    scheduler.abort(id);
    //    eprintln!("State = {:?}", *state.read().unwrap());
    //    assert_eq!(*state.read().unwrap(), 6);
    //    // Run the task every 3 seconds, 3 times
    //    let id = scheduler
    //        .schedule(task.clone(), every(3.seconds()).count(3))
    //        .unwrap();
    //    tokio::time::sleep(Duration::from_secs(10)).await;
    //    scheduler.abort(id);
    //    eprintln!("State = {:?}", *state.read().unwrap());
    //    assert_eq!(*state.read().unwrap(), 9);
    //    // Run the task every 1 minute and 2 seconds
    //    let id = scheduler
    //        .schedule(task, every(1.minutes()).and(2.seconds()))
    //        .unwrap();
    //    tokio::time::sleep(Duration::from_secs(120)).await;
    //    scheduler.pause(id);
    //    scheduler.resume(id);
    //    scheduler.abort(id);
    //    scheduler.pause(id);
    //    scheduler.resume(id);
    //    eprintln!("State = {:?}", *state.read().unwrap());
    //    assert_eq!(*state.read().unwrap(), 11);
    //    Ok(())
    //}

    #[tokio::test]
    async fn async_scheduler_single_task() -> Result<(), ()> {
        // Async Task
        let mut scheduler = Scheduler::build();
        #[derive(Clone, Task, Salt)]
        #[title = "MyAsyncTask"]
        struct MyAsyncTask {
            state: Arc<RwLock<u8>>,
        }
        let state = Arc::new(RwLock::new(0));
        let task = MyAsyncTask {
            state: state.clone(),
        };
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
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 1);
        // Run the task every second
        let id = scheduler
            .schedule(Box::new(task.clone()), every(1.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 6);
        // Run the task every 3 seconds, 3 times
        let id = scheduler
            .schedule(Box::new(task.clone()), every(3.seconds()).count(3))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 9);
        // Run the task every 1 minute and 2 seconds
        let id = scheduler
            .schedule(Box::new(task), every(1.minutes()).and(2.seconds()).count(1))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(120)).await;
        scheduler.pause(id);
        scheduler.resume(id);
        scheduler.abort(id);
        scheduler.pause(id);
        scheduler.resume(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 10);
        Ok(())
    }

    #[tokio::test]
    async fn async_reschedule_task() -> Result<(), ()> {
        // Async Task
        let mut scheduler = Scheduler::build();
        #[derive(Clone)]
        struct MyAsyncTask {
            state: Arc<RwLock<u8>>,
        }
        let state = Arc::new(RwLock::new(0));
        let task = MyAsyncTask {
            state: state.clone(),
        };
        impl Salt for MyAsyncTask {
            fn salt(&self) -> String {
                "MyAsyncTask".to_string()
            }
        }
        impl UniqueId for MyAsyncTask {}
        impl Task for MyAsyncTask {}
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
            .schedule(Box::new(task), every(1.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(3)).await;
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 3);
        let _new_id = scheduler.reschedule(id, every(5.seconds())).unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 5);
        Ok(())
    }

    #[tokio::test]
    async fn async_scheduler_multiple_instance_of_task() -> Result<(), ()> {
        let mut scheduler = Scheduler::build();

        #[derive(Debug, Clone)]
        struct MyAsyncTask {
            login: Arc<RwLock<String>>,
            state: Arc<RwLock<u8>>,
        }
        impl Salt for MyAsyncTask {
            fn salt(&self) -> String {
                self.login.read().unwrap().clone()
            }
        }
        impl UniqueId for MyAsyncTask {}
        impl Task for MyAsyncTask {}
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
            login: Arc::new(RwLock::new("pierre".to_string())),
            state: state.clone(),
        };
        let task2 = MyAsyncTask {
            login: Arc::new(RwLock::new("jean".to_string())),
            state: state.clone(),
        };
        let task3 = MyAsyncTask {
            login: Arc::new(RwLock::new("lucie".to_string())),
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
        #[derive(Debug, Clone)]
        struct MyAsyncTask {
            state: Arc<RwLock<u8>>,
        }
        let task = MyAsyncTask {
            state: Arc::new(RwLock::new(0)),
        };
        impl Salt for MyAsyncTask {
            fn salt(&self) -> String {
                format!("MyAsyncTask: {}", self.state.read().unwrap())
            }
        }
        impl UniqueId for MyAsyncTask {}
        impl Task for MyAsyncTask {
            fn title(&self) -> String {
                format!("First Task")
            }
        }
        impl AsyncTaskHandler for MyAsyncTask {
            fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_ + Sync>> {
                Box::pin(async move {
                    *self.state.write().unwrap() += 1;
                })
            }
        }
        #[derive(Debug, Clone)]
        struct MyAsyncTask2 {
            login: Arc<RwLock<String>>,
            age: Arc<RwLock<u32>>,
        }
        let task2 = MyAsyncTask2 {
            login: Arc::new(RwLock::new(String::from("Pierre"))),
            age: Arc::new(RwLock::new(25)),
        };
        impl Salt for MyAsyncTask2 {
            fn salt(&self) -> String {
                format!("MyAsyncTask2: {}", self.login.read().unwrap())
            }
        }
        impl UniqueId for MyAsyncTask2 {}
        impl Task for MyAsyncTask2 {
            fn title(&self) -> String {
                format!("Second Task with login {}", self.login.read().unwrap())
            }
        }
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
}
