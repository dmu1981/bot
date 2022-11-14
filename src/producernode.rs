use crate::node::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct ProducerState {
  millis: u64,
  tx: tokio::sync::broadcast::Sender<u64>,
  timer: Option<std::time::Instant>,
}

pub type ProducerNode = BotNode<ProducerState>;

impl ProducerNode {
  pub async fn on_new_state(&self) -> tokio::sync::broadcast::Receiver<u64> {
    self.state.lock().await.tx.subscribe()
  }
}

pub fn stop(mut state: tokio::sync::MutexGuard<'_, ProducerState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    println!("Producer node stopping...");
    sleep(Duration::from_millis(1500)).await;
    println!("Producer node stopped.");
    Ok(ThreadNext::Terminate)
  })
}

pub fn create(drop_tx: tokio::sync::broadcast::Receiver<()>,) -> ProducerNode {
  let (tx, rw) = tokio::sync::broadcast::channel(4);

  ProducerNode::new(
    "Producer".to_string(),
    drop_tx,
    ProducerState {
      millis: 100,
      tx,
      timer: None
    }
  )
}

pub fn update(duration: Duration, mut state: tokio::sync::MutexGuard<ProducerState>) -> NodeResult {
  println!("Producer update after {}ms", duration.as_millis());
  state.millis = state.millis + 100;
  
  state.tx.send(state.millis).unwrap();

  Ok(ThreadNext::Next)
}



pub fn init(mut _state: tokio::sync::MutexGuard<'_, ProducerState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    println!("Producer node initializing...");
    sleep(Duration::from_millis(1500)).await;
    println!("Producer node initialized.");
    Ok(ThreadNext::Terminate)
  })
}

pub fn async_update(mut state: tokio::sync::MutexGuard<'_, ProducerState>) -> DynFut<'_, NodeResult> {
  Box::pin(async move {
    let time_elapsed : Duration;
    match state.timer {
      None => { 
        time_elapsed = Duration::from_millis(0);
      },
      Some(timer) => {
        time_elapsed = timer.elapsed();
      }
    }

    // Restart the timer
    state.timer = Some(Instant::now()); 

    println!("Producer sleeping before he produces smth.");
    sleep(Duration::from_millis(100)).await;
    println!("Producer update after {}ms", time_elapsed.as_millis());
    state.millis = state.millis + 100;
  
    state.tx.send(state.millis).unwrap();

    Ok(ThreadNext::Next)
  })
}