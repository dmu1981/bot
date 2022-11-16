use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::timeout;

#[derive(Debug)]
pub enum ThreadNext {
    Next,
    Interval(std::time::Duration),
    Terminate,
}

pub type NodeHandle = JoinHandle<NodeResult>;
pub type Handles = Vec<NodeHandle>;

pub async fn execute(handles: Handles) {
    for result in futures::future::join_all(handles).await.iter() {
        result.as_ref().unwrap().as_ref().unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct ThreadError {
    pub msg: String,
}

pub type NodeResult = std::result::Result<ThreadNext, ThreadError>;
pub type State<'a, T> = tokio::sync::MutexGuard<'a, T>;
pub type DynFut<'lt, T> = Pin<Box<dyn 'lt + Send + Future<Output = T>>>;
pub type NodeList = Vec<Box<dyn Executor>>;

#[async_trait]
pub trait Executor {
    async fn init(&self) -> Handles;
    async fn run(&self) -> Handles;
    async fn stop(&self) -> Handles;
}

pub trait Node<T> {
    fn get_state_handle(&self) -> Arc<Mutex<T>>;
    fn get_drop_rx(&self) -> Receiver<()>;

    fn direct_once(&self, callback: fn(State<T>) -> NodeResult) -> JoinHandle<NodeResult>
    where
        T: Send + 'static,
    {
        let handle = self.get_state_handle();

        tokio::spawn(async move {
            let state = handle.lock().await;
            callback(state)
        })
    }

    fn once(&self, callback: fn(State<T>) -> DynFut<'_, NodeResult>) -> JoinHandle<NodeResult>
    where
        T: Send + 'static,
    {
        let handle = self.get_state_handle();

        tokio::spawn(async move {
            let state = handle.lock().await;
            let res = callback(state).await;

            if let Err(err) = &res {
                println!("Error: {}", err.msg);
            }

            res
        })
    }

    fn interval(
        &self,
        mut target_duration: std::time::Duration,
        callback: fn(tokio::sync::MutexGuard<T>) -> DynFut<'_, NodeResult>,
    ) -> JoinHandle<NodeResult>
    where
        T: Send + 'static,
    {
        let handle = self.get_state_handle();
        let mut drop_rx = self.get_drop_rx();
        let mut elapsed = Duration::from_millis(0);

        tokio::spawn(async move {
            loop {
                // Wait for the interval to pass
                let mut sleep_duration = Duration::from_millis(0);
                if elapsed < target_duration {
                    sleep_duration = target_duration - elapsed;
                }

                //println!("Sleep duration is {:?}", sleep_duration);

                if timeout(sleep_duration, drop_rx.recv()).await.is_ok() {
                    break;
                }

                // Start the timer
                let start_time = std::time::Instant::now();

                // Acquire a lock
                let state = handle.lock().await;

                // Callback
                match callback(state).await? {
                    ThreadNext::Next => {}
                    ThreadNext::Interval(dur) => {
                        target_duration = dur;
                    }
                    ThreadNext::Terminate => {
                        break;
                    }
                }

                // How much time elapsed
                elapsed = start_time.elapsed();
            }

            //println!("Dropping interval thread on {0}", node_name);
            Ok(ThreadNext::Terminate)
        })
    }

    fn direct_interval(
        &self,
        mut target_duration: std::time::Duration,
        callback: fn(tokio::sync::MutexGuard<T>) -> NodeResult, //callback: IntervalCallback<T>
    ) -> JoinHandle<NodeResult>
    where
        T: Send + 'static,
    {
        let handle = self.get_state_handle();
        let mut drop_rx = self.get_drop_rx();
        let mut elapsed = Duration::from_millis(0);

        tokio::spawn(async move {
            loop {
                // Wait for the interval to passd
                let mut sleep_duration = Duration::from_millis(0);
                if elapsed < target_duration {
                    sleep_duration = target_duration - elapsed;
                }

                // Wait for the interval to pass
                if timeout(sleep_duration, drop_rx.recv()).await.is_ok() {
                    break;
                }
                /*match timeout(sleep_duration, drop_rx.recv()).await {
                  Ok(_) => {
                    break;
                  }
                  _ => { }
                } */

                // Start the timer
                let start_time = std::time::Instant::now();

                // Acquire a lock
                let state = handle.lock().await;

                // Callback
                match callback(state)?//.await
        {
          ThreadNext::Next => { },
          ThreadNext::Interval(dur) => { target_duration = dur; }
          ThreadNext::Terminate => { break; }
        }

                // How much time elapsed
                elapsed = start_time.elapsed();
            }

            //println!("Dropping interval thread on {0}", node_name);
            Ok(ThreadNext::Terminate)
        })
    }

    fn direct_subscribe<U>(
        &self,
        mut rx: tokio::sync::broadcast::Receiver<U>,
        callback: fn(U, tokio::sync::MutexGuard<T>) -> NodeResult,
    ) -> JoinHandle<NodeResult>
    where
        T: Send + 'static,
        U: Clone + Send + 'static,
    {
        let handle = self.get_state_handle();
        let mut drop_rx = self.get_drop_rx();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                  value = rx.recv() => {
                    match value {
                      Ok(x) => {
                        let state = handle.lock().await;
                        match callback(x, state) {
                          Ok(ThreadNext::Next) => { },
                          Ok(ThreadNext::Interval(_)) => { panic!("Subscribe thread must not request an interval"); }
                          Ok(ThreadNext::Terminate) => { break; }
                          Err(_) => { panic!("Thread callback returns err"); }
                        }
                      },
                      Err(_) => {
                        println!("Recv failed");
                        break;
                      }
                    }
                  }
                  _ = drop_rx.recv() => {
                      break;
                  }
                }
            }

            //println!("Dropping subscribe thread on {0}", node_name);
            Ok(ThreadNext::Terminate)
        })
    }

    fn subscribe<U>(
        &self,
        mut rx: tokio::sync::broadcast::Receiver<U>,
        callback: fn(U, tokio::sync::MutexGuard<T>) -> DynFut<'_, NodeResult>,
    ) -> JoinHandle<NodeResult>
    where
        T: Send + 'static,
        U: Clone + Send + 'static,
    {
        let handle = self.get_state_handle();
        let mut drop_rx = self.get_drop_rx();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                  _ = drop_rx.recv() => {
                    break;
                  },
                  value = rx.recv() => {
                    match value {
                      Ok(x) => {
                        let state = handle.lock().await;
                        match callback(x, state).await? {
                          ThreadNext::Next => { },
                          _ => { break; }
                        }
                      },
                      Err(_) => {
                        //println!("Recv failed");
                        break;
                      }
                    }
                  }
                }
            }

            //println!("Dropping subscribe thread on {0}", node_name);
            Ok(ThreadNext::Terminate)
        })
    }
}

pub async fn execute_nodes(nodes: NodeList) {
    // Initialization
    println!("Initializing nodes....");
    let mut init_handles = Handles::new();
    for node in &nodes {
        init_handles.append(&mut node.init().await);
    }
    execute(init_handles).await;

    // Run all nodes
    println!("Running nodes....");
    let mut run_handles = Handles::new();
    for node in &nodes {
        run_handles.append(&mut node.run().await);
    }
    execute(run_handles).await;

    println!("Stopping nodes....");
    let mut stop_handles = Handles::new();
    for node in &nodes {
        stop_handles.append(&mut node.stop().await);
    }
    execute(stop_handles).await;
}
