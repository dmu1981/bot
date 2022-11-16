use crate::config::WheelExtrinsics;
use crate::config::*;
use crate::node::*;
use crate::wheelcontroller::WheelSpeed;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

//use
struct WheelControllerState {
    //wheel: String,
    url: String,
    client: reqwest::Client,
    extrinsics: WheelExtrinsics,
    speed: WheelSpeed,
    extrinsics_tx: Sender<WheelExtrinsics>,
    drop_tx: Sender<()>,
}

pub struct WheelControllerNode {
    pub wheelspeed_tx: Sender<crate::wheelcontroller::WheelSpeed>,
    pub extrinsics_rx: Receiver<WheelExtrinsics>,

    drop_rx: Receiver<()>,
    state: Arc<Mutex<WheelControllerState>>,
}

pub struct MyNode {
    pub controllers: Vec<WheelControllerNode>,
}

impl From<reqwest::Error> for ThreadError {
    fn from(err: reqwest::Error) -> ThreadError {
        ThreadError {
            msg: err.to_string(),
        }
    }
}

fn init_node(mut state: State<WheelControllerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        let result = reqwest::get(&state.url).await;

        match result {
            Ok(response) => {
                state.extrinsics =
                    serde_json::from_str(response.text().await.unwrap().as_str()).unwrap()
            }
            Err(err) => {
                state.drop_tx.send(()).unwrap();
                return Err(ThreadError::from(err));
            }
        }

        state.extrinsics_tx.send(state.extrinsics.clone()).unwrap();

        Ok(ThreadNext::Terminate)
    })
}

fn set_wheel_speed(
    wheel_speed: WheelSpeed,
    mut state: State<WheelControllerState>,
) -> DynFut<NodeResult> {
    Box::pin(async move {
        let flat_speed = match wheel_speed {
            WheelSpeed::Cw(x) => x,
            WheelSpeed::Ccw(x) => -x,
            WheelSpeed::Hold => 0.0,
        };
        state
            .client
            .post(&state.url)
            .body(serde_json::to_string(&flat_speed).unwrap())
            .send()
            .await
            .unwrap();
        state.speed = wheel_speed;

        Ok(ThreadNext::Next)
    })
}

fn stop(state: State<WheelControllerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        let flat_speed: f32 = 0.0;
        state
            .client
            .post(&state.url)
            .body(serde_json::to_string(&flat_speed).unwrap())
            .send()
            .await
            .unwrap();

        Ok(ThreadNext::Terminate)
    })
}

fn create(
    wheel: &crate::config::Wheel,
    name: &String,
    url: &str,
    drop_tx: Sender<()>,
) -> WheelControllerNode {
    let (tx, _) = tokio::sync::broadcast::channel::<WheelSpeed>(4);
    let (tx2, _) = tokio::sync::broadcast::channel::<WheelExtrinsics>(1);

    WheelControllerNode {
        drop_rx: drop_tx.subscribe(),
        wheelspeed_tx: tx,
        extrinsics_rx: tx2.subscribe(),
        state: Arc::new(Mutex::new(WheelControllerState {
            //wheel: name.clone(),
            drop_tx,
            client: reqwest::Client::new(),
            extrinsics: WheelExtrinsics {
                pivot: wheel.pivot.clone(),
                forward: wheel.forward.clone(),
            },
            url: url.to_owned() + "/wheel/" + name,
            speed: WheelSpeed::Hold,
            //tx,
            extrinsics_tx: tx2,
        })),
    }
}

pub fn create_all(config: &Config, drop_tx: Sender<()>) -> MyNode {
    let mut controller = MyNode {
        controllers: Vec::<WheelControllerNode>::new(),
    };

    for wheel in config.wheels.iter() {
        controller.controllers.push(create(
            wheel,
            &wheel.name,
            &config.simulation.url,
            drop_tx.clone(),
        ));
    }

    controller
}
/*
pub async fn run(wheel_controller: &Vec<WheelControllerNode>) -> Handles {
    let mut handles = Handles::new();
    for wc in wheel_controller {
        handles.push(wc.subscribe(wc.wheelspeed_tx.subscribe(), set_wheel_speed));
    }
    handles
}

pub async fn stop(wheel_controller: &Vec<WheelControllerNode>) -> Handles {
    let mut handles = Handles::new();
    for wc in wheel_controller {
        handles.push(wc.once(reset_pins));
    }
    handles
}*/

#[async_trait]
impl Executor for MyNode {
    async fn init(&self) -> Handles {
        let mut handles = Handles::new();
        for wc in &self.controllers {
            handles.push(wc.once(init_node));
        }
        handles
    }

    async fn run(&self) -> Handles {
        let mut handles = Handles::new();
        for wc in &self.controllers {
            handles.push(wc.subscribe(wc.wheelspeed_tx.subscribe(), set_wheel_speed));
        }
        handles
    }

    async fn stop(&self) -> Handles {
        let mut handles = Handles::new();
        for wc in &self.controllers {
            handles.push(wc.once(stop));
        }
        handles
    }
}

impl Node<WheelControllerState> for WheelControllerNode {
    fn get_state_handle(&self) -> Arc<Mutex<WheelControllerState>> {
        self.state.clone()
    }

    fn get_drop_rx(&self) -> Receiver<()> {
        self.drop_rx.resubscribe()
    }
}
