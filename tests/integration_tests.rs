#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
        time::Duration,
    };

    use pulsync::{prelude::*, task::Salt};

    #[tokio::test]
    async fn sync_scheduler_single_task() -> Result<(), ()> {
        // Sync task
        let mut scheduler: Arc<RwLock<HashMap<TaskId, SyncTask<_>>>> = Scheduler::new();
        #[derive(Debug, Clone)]
        struct MySyncTask {
            state: Arc<RwLock<u8>>,
        }
        let state = Arc::new(RwLock::new(0));
        let task = MySyncTask {
            state: state.clone(),
        };
        impl Salt for MySyncTask {
            fn salt(&self) -> String {
                "MySyncTask".to_string()
            }
        }
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
        tokio::time::sleep(Duration::from_secs(1)).await;
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 1);
        // Run the task every second
        let id = scheduler
            .schedule(task.clone(), every(1.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 6);
        // Run the task every 3 seconds, 3 times
        let id = scheduler
            .schedule(task.clone(), every(3.seconds()).count(3))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 9);
        // Run the task every 1 minute and 2 seconds
        let id = scheduler
            .schedule(task, every(1.minutes()).and(2.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(120)).await;
        scheduler.pause(id);
        scheduler.resume(id);
        scheduler.abort(id);
        scheduler.pause(id);
        scheduler.resume(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 11);
        Ok(())
    }

    #[tokio::test]
    async fn async_scheduler_single_task() -> Result<(), ()> {
        // Async Task
        let mut scheduler = Scheduler::new();
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
        let id = scheduler
            .schedule(task.clone(), every(1.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 6);
        // Run the task every 3 seconds, 3 times
        let id = scheduler
            .schedule(task.clone(), every(3.seconds()).count(3))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
        scheduler.abort(id);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 9);
        // Run the task every 1 minute and 2 seconds
        let id = scheduler
            .schedule(task, every(1.minutes()).and(2.seconds()).count(1))
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
        let mut scheduler = Scheduler::new();
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
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                *state += 1;
            }
        }

        // Run the task once
        let id = scheduler.schedule(task, every(1.seconds())).unwrap();
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
        let mut scheduler = Scheduler::new();

        #[derive(Debug, Clone)]
        struct MyAsyncTask {
            login: String,
            state: Arc<RwLock<u8>>,
        }
        impl Salt for MyAsyncTask {
            fn salt(&self) -> String {
                self.login.clone()
            }
        }
        impl UniqueId for MyAsyncTask {}
        impl Task for MyAsyncTask {}
        impl AsyncTaskHandler for MyAsyncTask {
            async fn run(&self) {
                let mut state = self.state.write().unwrap();
                eprintln!("Message from {} with value {}", self.login, *state);
                *state += 1;
            }
        }

        let state = Arc::new(RwLock::new(0));
        let task = MyAsyncTask {
            login: "pierre".to_string(),
            state: state.clone(),
        };
        let task2 = MyAsyncTask {
            login: "jean".to_string(),
            state: state.clone(),
        };
        let task3 = MyAsyncTask {
            login: "lucie".to_string(),
            state: state.clone(),
        };
        // Run the task once
        scheduler.run(task.clone());
        tokio::time::sleep(Duration::from_secs(1)).await;
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 1);
        // Run the tasks every 5 seconds
        let id1 = scheduler
            .schedule(task.clone(), every(5.seconds()))
            .unwrap();
        let id2 = scheduler
            .schedule(task2.clone(), every(5.seconds()))
            .unwrap();
        let id3 = scheduler
            .schedule(task3.clone(), every(5.seconds()))
            .unwrap();
        tokio::time::sleep(Duration::from_secs(15)).await;
        scheduler.abort(id1);
        scheduler.abort(id2);
        scheduler.abort(id3);
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 10);
        Ok(())
    }
}
