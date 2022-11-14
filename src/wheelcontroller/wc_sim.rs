use tokio::sync::broadcast::{Sender, Receiver};
use crate::node::*;
use crate::config::*;
use crate::wheelcontroller::WheelSpeed;
use crate::config::WheelExtrinsics;

pub struct WheelControllerState {
  //wheel: String,
  url: String,
  client: reqwest::Client,
  extrinsics: WheelExtrinsics,
  speed: WheelSpeed,
  tx: Sender<crate::wheelcontroller::WheelSpeed>,
  extrinsics_tx: Sender<WheelExtrinsics>,
}

pub type WheelControllerNode = BotNode<WheelControllerState>;

pub fn init_node(mut state: State<WheelControllerState>) -> DynFut<NodeResult>
{
  Box::pin(async move {
    state.extrinsics = serde_json::from_str(
      &reqwest::get(&state.url).await.unwrap().text().await.unwrap()
    ).unwrap();

    state.extrinsics_tx.send(state.extrinsics.clone()).unwrap();

    Ok(ThreadNext::Terminate)
  })
}

pub fn set_wheel_speed(wheel_speed: WheelSpeed, mut state: State<WheelControllerState>) -> DynFut<NodeResult>
{
  Box::pin(async move {
    
    let flat_speed = match wheel_speed {
      WheelSpeed::Cw(x) => x,
      WheelSpeed::Ccw(x) => -x,
      WheelSpeed::Hold => 0.0
    };
    state.client.post(&state.url).body(serde_json::to_string(&flat_speed).unwrap()).send().await.unwrap();
    state.speed = wheel_speed;

    Ok(ThreadNext::Next)
  })
}

pub fn reset_pins(_state: State<WheelControllerState>) -> DynFut<NodeResult>
{
  Box::pin(async move {
    Ok(ThreadNext::Terminate)
  })
}

pub fn toggle_pins(_state: State<WheelControllerState>) -> NodeResult
{
  Ok(ThreadNext::Terminate)
}

pub async fn wheel_speed_tx(wc: &WheelControllerNode) -> Sender<WheelSpeed> {
  wc.state.lock().await.tx.clone()
}

pub async fn wheel_extrinsics_tx(wc: &WheelControllerNode) -> Receiver<WheelExtrinsics> {
  wc.state.lock().await.extrinsics_tx.subscribe()
}

fn create( 
  wheel: &crate::config::Wheel,
  name: &String,
  url: &str,
  drop_tx: Receiver<()>) -> WheelControllerNode {
    let (tx, _) = tokio::sync::broadcast::channel::<WheelSpeed>(4);
    let (tx2, _) = tokio::sync::broadcast::channel::<WheelExtrinsics>(1);
                                                       
    WheelControllerNode::new(
      "Wheel Controller (Simulation)".to_string(), 
      drop_tx, 
      WheelControllerState {
        //wheel: name.clone(),
        client: reqwest::Client::new(),
        extrinsics: WheelExtrinsics {
          pivot: wheel.pivot.clone(),
          forward: wheel.forward.clone(),
        },
        url: url.to_owned() + "/wheel/" + name,
        speed: WheelSpeed::Hold,
        tx,
        extrinsics_tx: tx2,
      }
    )
}

pub fn create_all(
  config: &Config,
  drop_tx: &Sender<()>) -> Vec<WheelControllerNode> {
    let mut controller = Vec::<WheelControllerNode>::new();

    for wheel in config.wheels.iter() {
      controller.push(create(wheel, &wheel.name, &config.simulation.url, drop_tx.subscribe()));
    }

    controller
  }

pub async fn init(wheel_controller : &Vec::<WheelControllerNode>) -> Handles                     
{
  let mut handles = Handles::new();
  for wc in wheel_controller {
    handles.push(wc.once(init_node));
  }
  handles
}

pub async fn run(wheel_controller : &Vec::<WheelControllerNode>) -> Handles 
{
  let mut handles = Handles::new();
  for wc in wheel_controller {
    handles.push(wc.subscribe(wheel_speed_tx(wc).await.subscribe(), set_wheel_speed));
  }
  handles
}

pub async fn stop(wheel_controller : &Vec::<WheelControllerNode>) -> Handles
{
  let mut handles = Handles::new();
  for wc in wheel_controller {
    handles.push(wc.once(reset_pins));
  }
  handles
}



