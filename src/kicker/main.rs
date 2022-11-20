use crate::config::Config;
use crate::node::*;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

struct KickerState {
    url: String,
    client: reqwest::Client,
}

pub struct KickerNode {
    pub kick_tx: Sender<bool>,

    drop_rx: Receiver<()>,
    state: Arc<Mutex<KickerState>>,
    kick_rx: Receiver<bool>,
}

fn on_kick(kick: bool, state: State<KickerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        if kick {
            state
                .client
                .post(&state.url)
                .body("empty body".to_owned())
                .send()
                .await
                .unwrap();
        }

        Ok(ThreadNext::Next)
    })
}

pub fn create(config: &Config, drop_tx: Sender<()>) -> KickerNode {
    let (tx, rx) = tokio::sync::broadcast::channel::<bool>(4);

    KickerNode {
        drop_rx: drop_tx.subscribe(),
        kick_tx: tx,
        kick_rx: rx,
        state: Arc::new(Mutex::new(KickerState {
            client: reqwest::Client::new(),
            url: config.simulation.url.to_owned() + "/kicker",
        })),
    }
}

impl Node<KickerState> for KickerNode {
    fn get_state_handle(&self) -> Arc<Mutex<KickerState>> {
        self.state.clone()
    }

    fn get_drop_rx(&self) -> Receiver<()> {
        self.drop_rx.resubscribe()
    }
}

#[async_trait]
impl Executor for KickerNode {
    async fn init(&self) -> Handles {
        vec![]
    }

    async fn run(&self) -> Handles {
        vec![self.subscribe(self.kick_rx.resubscribe(), on_kick)]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
