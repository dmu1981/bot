use crate::config::WheelExtrinsics;
use crate::config::*;
use crate::math::clamp;
use crate::node::*;
//use crate::wheelcontroller::WheelSpeed;
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
    speed: f32,
    extrinsics_tx: Sender<WheelExtrinsics>,
    drop_tx: Sender<()>,
}

pub struct WheelControllerNode {
    pub wheelspeed_tx: Sender<f32>,
    pub extrinsics_rx: Receiver<WheelExtrinsics>,
    drop_rx: Receiver<()>,
    state: Arc<Mutex<WheelControllerState>>,
}

pub struct MyNode {
    pub controllers: Vec<WheelControllerNode>,
    bot_spawned_rx: Receiver<bool>,
}

impl From<reqwest::Error> for ThreadError {
    fn from(err: reqwest::Error) -> ThreadError {
        ThreadError {
            msg: err.to_string(),
        }
    }
}

fn init_node(spawned: bool, mut state: State<WheelControllerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        if !spawned {
            return Ok(ThreadNext::Next);
        }

        //println!("Requesting extrinsics");
        //tokio::time::sleep(Duration::from_millis(2500)).await;

        let result = reqwest::get(&state.url).await;

        match result {
            Ok(response) => {
                let t = response.text().await.unwrap();
                //println!("{}", t);
                state.extrinsics = serde_json::from_str(t.as_str()).unwrap()
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
    mut wheel_speed: f32,
    mut state: State<WheelControllerState>,
) -> DynFut<NodeResult> {
    Box::pin(async move {
        wheel_speed = clamp(wheel_speed, -1.0, 1.0);

        state
            .client
            .post(&state.url)
            .body(serde_json::to_string(&wheel_speed).unwrap())
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
    let (tx, _) = tokio::sync::broadcast::channel::<f32>(160);
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
                pivot: wheel.pivot,
                forward: wheel.forward,
            },
            url: url.to_owned() + "/wheel/" + name,
            speed: 0.0,
            extrinsics_tx: tx2,
        })),
    }
}

pub fn create_all(config: &Config, bot_spawned_rx: Receiver<bool>, drop_tx: Sender<()>) -> MyNode {
    let mut controller = MyNode {
        controllers: Vec::<WheelControllerNode>::new(),
        bot_spawned_rx,
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

#[async_trait]
impl Executor for MyNode {
    async fn init(&self) -> Handles {
        let mut handles = Handles::new();
        for wc in &self.controllers {
            handles.push(wc.subscribe(self.bot_spawned_rx.resubscribe(), init_node));
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
