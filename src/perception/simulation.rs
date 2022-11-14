use tokio::sync::broadcast;
use crate::node::*;
use crate::config::*;

pub struct PerceptionState {
}

pub type PerceptionNode = BotNode<PerceptionState>;

pub fn init(_state: tokio::sync::MutexGuard<'_, PerceptionState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    Ok(ThreadNext::Terminate)
  })
}

pub fn update(_state: tokio::sync::MutexGuard<'_, PerceptionState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    Ok(ThreadNext::Next)
  })
}

pub async fn create(
  _config: &Config,
  drop_tx: broadcast::Receiver<()>) -> PerceptionNode 
  {
    PerceptionNode::new(
      "Perception (Simulation)".to_string(),
      drop_tx,
      PerceptionState {

      })
  }
  