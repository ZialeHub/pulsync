#[cfg(test)]
mod test {
    use std::{
        sync::{Arc, RwLock},
        time::Duration,
    };

    use pulsync::prelude::*;

    #[tokio::test]
    async fn sync_scheduler_single_task() -> Result<(), ()> {
        // Sync task
        let mut scheduler = Scheduler::new();
        #[derive(Debug, Clone)]
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
        tokio::time::sleep(Duration::from_secs(1)).await;
        scheduler.abort::<MySyncTask>();
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 1);
        // Run the task every second
        scheduler.schedule(task.clone(), every(1.seconds()));
        tokio::time::sleep(Duration::from_secs(5)).await;
        scheduler.abort::<MySyncTask>();
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 6);
        // Run the task every 3 seconds, 3 times
        scheduler.schedule(task.clone(), every(3.seconds()).count(3));
        tokio::time::sleep(Duration::from_secs(10)).await;
        scheduler.abort::<MySyncTask>();
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 9);
        // Run the task every 1 minute and 2 seconds
        scheduler.schedule(task, every(1.minutes()).and(2.seconds()));
        tokio::time::sleep(Duration::from_secs(120)).await;
        scheduler.pause::<MySyncTask>();
        scheduler.resume::<MySyncTask>();
        scheduler.abort::<MySyncTask>();
        scheduler.pause::<MySyncTask>();
        scheduler.resume::<MySyncTask>();
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
        scheduler.abort::<MyAsyncTask>();
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 6);
        // Run the task every 3 seconds, 3 times
        scheduler.schedule(task.clone(), every(3.seconds()).count(3));
        tokio::time::sleep(Duration::from_secs(10)).await;
        scheduler.abort::<MyAsyncTask>();
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 9);
        // Run the task every 1 minute and 2 seconds
        scheduler.schedule(task, every(1.minutes()).and(2.seconds()).count(1));
        tokio::time::sleep(Duration::from_secs(120)).await;
        scheduler.pause::<MyAsyncTask>();
        scheduler.resume::<MyAsyncTask>();
        scheduler.abort::<MyAsyncTask>();
        scheduler.pause::<MyAsyncTask>();
        scheduler.resume::<MyAsyncTask>();
        eprintln!("State = {:?}", *state.read().unwrap());
        assert_eq!(*state.read().unwrap(), 10);
        Ok(())
    }
}
