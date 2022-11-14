use crate::node::*;
use std::time::Duration;
use tokio::time::sleep;

pub struct TestNodeState {
  pub internal : u64,
}

pub type TestNode = BotNode<TestNodeState>;

pub fn create(drop_tx: tokio::sync::broadcast::Receiver<()>) -> TestNode {
  TestNode::new(
    "Consumer".to_string(), 
    drop_tx, 
    TestNodeState {
      internal: 0
    }
  )
}



pub fn init(_state: tokio::sync::MutexGuard<'_, TestNodeState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    println!("Test node initializing...");
    sleep(Duration::from_millis(1500)).await;
    println!("Test node initialized.");
    Ok(ThreadNext::Terminate)
  })
}

pub fn stop(mut state: tokio::sync::MutexGuard<'_, TestNodeState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    println!("Test node stopping...");
    sleep(Duration::from_millis(1500)).await;
    println!("Test node stopped.");
    Ok(ThreadNext::Terminate)
  })
}

pub fn update_state(msg: u64, mut state: tokio::sync::MutexGuard<'_, TestNodeState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    println!("Consumer sleeping before he outputs smth.");
    sleep(Duration::from_millis(50)).await;

    state.internal = msg;
    println!("Internal state is {0}", state.internal);

    Ok(ThreadNext::Next)
  })
}

pub fn print_state(msg: bool, mut state: tokio::sync::MutexGuard<TestNodeState>) -> NodeResult
{
  println!("Internal state is {0}", state.internal);
  Ok(ThreadNext::Next)
}