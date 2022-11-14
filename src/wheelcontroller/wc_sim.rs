use tokio::sync::broadcast;
use crate::node::*;
use crate::config::*;
use crate::wheelcontroller::WheelSpeed;
use serde::{Serialize, Deserialize};
use serde_json::*;
use crate::math::Vec2;
use reqwest::*;




pub struct WheelControllerState {
  wheel: String,
  url: String,
  client: reqwest::Client,
  extrinsics: crate::wheelcontroller::WheelExtrinsics,
  speed: WheelSpeed,
  tx: tokio::sync::broadcast::Sender<crate::wheelcontroller::WheelSpeed>,
  extrinsics_tx: tokio::sync::broadcast::Sender<crate::wheelcontroller::WheelExtrinsics>,
}

pub type WheelControllerNode = BotNode<WheelControllerState>;

pub fn init(mut state: tokio::sync::MutexGuard<'_, WheelControllerState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    state.extrinsics = serde_json::from_str(
      &reqwest::get(&state.url).await.unwrap().text().await.unwrap()
    ).unwrap();

    state.extrinsics_tx.send(state.extrinsics.clone()).unwrap();

    Ok(ThreadNext::Terminate)
  })
}

pub fn set_wheel_speed(wheelSpeed: WheelSpeed, mut state: tokio::sync::MutexGuard<'_, WheelControllerState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    
    let flatSpeed = match wheelSpeed {
      WheelSpeed::CW(x) => x,
      WheelSpeed::CCW(x) => -x,
      WheelSpeed::Hold => 0.0
    };
    state.client.post(&state.url).body(serde_json::to_string(&flatSpeed).unwrap()).send().await.unwrap();
    state.speed = wheelSpeed;

    Ok(ThreadNext::Next)
  })
}

pub fn reset_pins(mut state: tokio::sync::MutexGuard<'_, WheelControllerState>) -> DynFut<'_, NodeResult>
{
  Box::pin(async move {
    Ok(ThreadNext::Terminate)
  })
}

pub fn toggle_pins(mut state: tokio::sync::MutexGuard<'_, WheelControllerState>) -> NodeResult
{
  Ok(ThreadNext::Terminate)
}

pub async fn wheel_speed_tx(wc: &WheelControllerNode) -> tokio::sync::broadcast::Sender<crate::wheelcontroller::WheelSpeed> {
  wc.state.lock().await.tx.clone()
}

pub async fn wheel_extrinsics_tx(wc: &WheelControllerNode) -> tokio::sync::broadcast::Receiver<crate::wheelcontroller::WheelExtrinsics> {
  wc.state.lock().await.extrinsics_tx.subscribe()
}

fn create( 
  wheel: &crate::config::Wheel,
  name: &String,
  url: &String,
  drop_tx: broadcast::Receiver<()>) -> WheelControllerNode {
    let (tx, rx) = tokio::sync::broadcast::channel::<crate::wheelcontroller::WheelSpeed>(4);
    let (tx2, rx2) = tokio::sync::broadcast::channel::<crate::wheelcontroller::WheelExtrinsics>(1);
                                                       
    WheelControllerNode::new(
      "Wheel Controller (Simulation)".to_string(), 
      drop_tx, 
      WheelControllerState {
        wheel: name.clone(),
        client: reqwest::Client::new(),
        extrinsics: crate::wheelcontroller::WheelExtrinsics {
          pivot: wheel.pivot.clone(),
          forward: wheel.forward.clone(),
        },
        url: url.clone() + "/wheel/" + name,
        speed: WheelSpeed::Hold,
        tx,
        extrinsics_tx: tx2,
      }
    )
}

pub fn create_all(
  config: &Config,
  drop_tx: &broadcast::Sender<()>) -> Vec<WheelControllerNode> {
    let mut controller = Vec::<WheelControllerNode>::new();

    for wheel in config.wheels.iter() {
      controller.push(create(wheel, &wheel.name, &config.simulation.url, drop_tx.subscribe()));
    }

    controller
  }




