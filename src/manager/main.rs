use crate::config::Config;
use crate::node::*;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

const API_VERSION: u32 = 1;

struct ManagerState {
    api_url: String,
    reset_url: String,
    client: reqwest::Client,
    bot_spawned_tx: Sender<bool>,
}

pub struct ManagerNode {
    drop_rx: Receiver<()>,
    state: Arc<Mutex<ManagerState>>,
    pub bot_spawned_rx: Receiver<bool>,
}

#[derive(Deserialize)]
struct APIVersion {
    version: u32,
}

async fn send_reset_signal(url: &String, client: &reqwest::Client) {
    client.post(url).send().await.unwrap();
}

fn reset_sim(reset: bool, state: State<ManagerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        if reset {
            send_reset_signal(&state.reset_url, &state.client).await;
        }

        Ok(ThreadNext::Next)
    })
}

fn api_check(state: State<ManagerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        let response = state.client.get(&state.api_url).send().await.unwrap();

        let version: APIVersion =
            serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
        if version.version != API_VERSION {
            return Err(ThreadError {
                msg: "API Version mismatch. Consider updating the simulation.".to_string(),
            });
        }

        send_reset_signal(&state.reset_url, &state.client).await;

        state.bot_spawned_tx.send(true).unwrap();

        Ok(ThreadNext::Terminate)
    })
}

pub fn create(config: &Config, drop_tx: Sender<()>) -> ManagerNode {
    let (tx, rx) = tokio::sync::broadcast::channel::<bool>(1);

    ManagerNode {
        drop_rx: drop_tx.subscribe(),
        bot_spawned_rx: rx,
        state: Arc::new(Mutex::new(ManagerState {
            client: reqwest::Client::new(),
            bot_spawned_tx: tx,
            api_url: config.simulation.url.to_owned() + "/api",
            reset_url: config.simulation.url.to_owned() + "/reset",
        })),
    }
}

impl Node<ManagerState> for ManagerNode {
    fn get_state_handle(&self) -> Arc<Mutex<ManagerState>> {
        self.state.clone()
    }

    fn get_drop_rx(&self) -> Receiver<()> {
        self.drop_rx.resubscribe()
    }
}

#[async_trait]
impl Executor for ManagerNode {
    async fn init(&self) -> Handles {
        vec![self.once(api_check)]
    }

    async fn run(&self) -> Handles {
        vec![]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
