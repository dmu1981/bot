use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;
use std::time::{Duration};
use tokio::time::timeout;
use std::pin::Pin;
use std::future::Future;

pub struct BotNode<T>{
  name : String,
  drop : tokio::sync::broadcast::Receiver<()>,
  pub state : Arc<Mutex<T>>,
}

pub enum ThreadNext
{
  Next,
  Interval(std::time::Duration),
  Terminate
}

pub type NodeHandle = JoinHandle<NodeResult>;

pub type Handles = Vec::<NodeHandle>;

pub async fn execute(handles : Handles) {
  for result in futures::future::join_all(handles).await.iter() {
    result.as_ref().unwrap().as_ref().unwrap();
  }
}

#[derive(Debug, Clone)]
pub struct ThreadError
{
  pub msg: String,
}

pub type NodeResult = std::result::Result<ThreadNext, ThreadError>;
pub type State<'a, T> = tokio::sync::MutexGuard<'a, T>;

pub type DynFut<'lt, T> = 
  Pin<Box<dyn 'lt + Send + Future<Output = T>>>;

impl<T> BotNode<T> {
  pub fn new(name: String, drop_tx: Receiver<()>, state: T) -> BotNode<T> {
    BotNode {
      name,
      drop: drop_tx,
      state: Arc::new(Mutex::new(state)),
    }
  }

  pub fn direct_once(&self, 
                callback: fn(State<T>) -> NodeResult) 
                -> JoinHandle<NodeResult> 
               where T: Send + 'static,
  {
    let handle = self.state.clone();
    
    tokio::spawn(async move {
      let state = handle.lock().await;
      callback(state)
    })
  }

  pub fn once(&self, 
                     callback: fn(State<T>) -> DynFut<'_, NodeResult>)
                      -> JoinHandle<NodeResult> 
               where T: Send + 'static,
  {
    let handle = self.state.clone();

    tokio::spawn(async move {
      let state = handle.lock().await;
      callback(state).await      
    })
  }

  pub fn interval(&self, 
                  mut target_duration: std::time::Duration,
                  callback: fn(tokio::sync::MutexGuard<T>) -> DynFut<'_, NodeResult>
                  ) 
                  -> JoinHandle<NodeResult> 
                  where T: Send + 'static,
  {
    let handle = self.state.clone();
    let mut drop_rx = self.drop.resubscribe();
    let mut elapsed = Duration::from_millis(0);

    tokio::spawn(async move {
      loop
      { 
        // Wait for the interval to pass
        let mut sleep_duration = Duration::from_millis(0);
        if elapsed < target_duration {
          sleep_duration = target_duration - elapsed;
        }

        if timeout(sleep_duration, drop_rx.recv()).await.is_ok() {
          break;
        }       
        
        // Start the timer
        let start_time = std::time::Instant::now();

        // Acquire a lock        
        let state = handle.lock().await;

        // Callback
        match callback(state).await?
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

  pub fn direct_interval(&self, 
                  mut target_duration: std::time::Duration,
                  callback: fn(tokio::sync::MutexGuard<T>) -> NodeResult
                  //callback: IntervalCallback<T>
                  ) 
                  -> JoinHandle<NodeResult> 
                  where T: Send + 'static,
  {
    let handle = self.state.clone();
    let mut drop_rx = self.drop.resubscribe();
    let mut elapsed = Duration::from_millis(0);

    tokio::spawn(async move {
      loop
      {
        // Wait for the interval to pass
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

  pub fn direct_subscribe<U>(&self,
             mut rx: tokio::sync::broadcast::Receiver<U>,
             callback: fn(U, tokio::sync::MutexGuard<T>) -> NodeResult
            ) -> JoinHandle<NodeResult> 
            where T: Send + 'static, U: Clone + Send + 'static
  {
    let handle = self.state.clone();
    let mut drop_rx = self.drop.resubscribe();

    tokio::spawn(async move {
      loop
      {
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

  pub fn subscribe<U>(&self,
             mut rx: tokio::sync::broadcast::Receiver<U>,
             callback: fn(U, tokio::sync::MutexGuard<T>) -> DynFut<'_, NodeResult>
            ) -> JoinHandle<NodeResult> 
            where T: Send + 'static, U: Clone + Send + 'static
  {
    let handle = self.state.clone();
    let mut drop_rx = self.drop.resubscribe();

    tokio::spawn(async move {
      loop
      {
        tokio::select! {
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
          _ = drop_rx.recv() => {
              break;
          }
        }        
      }

      //println!("Dropping subscribe thread on {0}", node_name);
      Ok(ThreadNext::Terminate)
    })
  }

  

}

