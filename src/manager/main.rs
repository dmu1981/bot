use crate::config::Config;
use crate::node::*;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

const API_VERSION : u32 = 1;

struct ManagerState {
    api_url: String,
    client: reqwest::Client,
}

pub struct ManagerNode {
    drop_rx: Receiver<()>,
    state: Arc<Mutex<ManagerState>>,
}

#[derive(Deserialize)]
struct APIVersion {
    version: u32,
}

fn on_api_check(state: State<ManagerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        let response = state.client.get(&state.api_url).send().await.unwrap();

        let version: APIVersion =
            serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
        if version.version != API_VERSION {
            Err(ThreadError {
                msg: "API Version mismatch. Consider updating the simulation.".to_string(),
            })
        } else {
            Ok(ThreadNext::Terminate)
        }
    })
}

pub fn create(config: &Config, drop_tx: Sender<()>) -> ManagerNode {
    ManagerNode {
        drop_rx: drop_tx.subscribe(),
        state: Arc::new(Mutex::new(ManagerState {
            client: reqwest::Client::new(),
            api_url: config.simulation.url.to_owned() + "/api",
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
        vec![self.once(on_api_check)]
    }

    async fn run(&self) -> Handles {
        vec![]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
