use crate::config::Config;
use crate::node::*;
use async_trait::async_trait;
use serde::Deserialize;
use std::net::{TcpListener, Shutdown};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;
use std::io::{Write, Read};

const API_VERSION: u32 = 2;

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

    reset_sim_rx: Receiver<bool>,
    pub reset_sim_tx: Sender<bool>,
}

#[derive(Deserialize)]
struct APIVersion {
    version: u32,
}

async fn send_reset_signal(url: &String, client: &reqwest::Client) {
    client.post(url).body("1.0".as_bytes()).send().await.unwrap();
}

fn reset_sim(reset: bool, state: State<ManagerState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        if reset {
            send_reset_signal(&state.reset_url, &state.client).await;
        }

        Ok(ThreadNext::Next)
    })
}

fn health_check(state: State<ManagerState>) -> DynFut<NodeResult> {
  tokio::spawn(async move {
    return;
    let listener = TcpListener::bind("127.0.0.1:3333").unwrap();

    println!("Echo Listening on port 3333...\n");

    loop {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0; 1024];        
        stream.read(&mut buffer).unwrap();
        let content = match std::str::from_utf8(&mut buffer) {
          Ok(v) => v,
          Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
      };

      stream.write_all("HTTP/1.1 200 OK\n\n".as_bytes()).unwrap();
      stream.write_all(content.as_bytes()).unwrap();

      stream.shutdown(Shutdown::Both).unwrap();
    }
  });

  Box::pin(async move {
    Ok(ThreadNext::Terminate)
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

        tokio::time::sleep(Duration::from_millis(250)).await;

        state.bot_spawned_tx.send(true).unwrap();

        Ok(ThreadNext::Terminate)
    })
}

pub fn create(config: &Config, drop_tx: Sender<()>) -> ManagerNode {
    let (tx, rx) = tokio::sync::broadcast::channel::<bool>(1);
    let (tx2, rx2) = tokio::sync::broadcast::channel::<bool>(1);

    ManagerNode {
        drop_rx: drop_tx.subscribe(),
        bot_spawned_rx: rx,
        reset_sim_tx: tx2,
        reset_sim_rx: rx2,
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
        vec![self.once(api_check), self.once(health_check)]
    }

    async fn run(&self) -> Handles {
        vec![self.subscribe(self.reset_sim_rx.resubscribe(), reset_sim)]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
