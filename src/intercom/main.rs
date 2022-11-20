use crate::config::Config;
use crate::math::Vec2;
use crate::node::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IntercomMessage {
  Position(IntercomPosition),
  ModeTransition(IntercomMode),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BehaviorMode {
  Offense,
  Defense
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntercomPosition {
    pub ball: Vec2,
    pub own_goal: Vec2,
    pub target_goal: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntercomMode {
    pub mode: BehaviorMode
}

struct IntercomState {
    master: bool,
    port: u32,
    send_message_rx: Receiver<IntercomMessage>,
    drop_rx: Receiver<()>,
}

pub struct IntercomNode {
    pub send_message_tx: Sender<IntercomMessage>,
    state: Arc<Mutex<IntercomState>>,
    drop_rx: Receiver<()>,
}

impl From<std::io::Error> for crate::node::ThreadError {
    fn from(err: std::io::Error) -> crate::node::ThreadError {
        ThreadError {
            msg: err.to_string(),
        }
    }
}

use serde_json::error::Category;

enum ComResult {
    Closed,
    Drop,
}

async fn handle_com(mut stream: TcpStream, state: &mut State<'_, IntercomState>) -> ComResult {
    let mut buf = vec![0; 1024];
    let mut drop_rx = state.drop_rx.resubscribe();

    let mut remainder: String = "".to_owned();

    loop {
        select! {
          msg = state.send_message_rx.recv() => {
            match msg {
              Ok(inner) => {
                let body = serde_json::to_string(&inner).unwrap() + "\0";
                let n = stream.write(body.as_bytes()).await.unwrap();
                if n != body.as_bytes().len() {
                  panic!("Could not send all bytes");
                }
              },
              Err(err) => {
                match err {
                  RecvError::Closed => {
                    return ComResult::Closed;
                  },
                  RecvError::Lagged(cnt) => {
                    println!("Intercom lagged {} messages, silently ignoring this.", cnt);
                    continue;
                  }
                }
              }
            }

          },
          res = stream.read(&mut buf) => {
            if res.is_err() {
              println!("Intercom reset connection");
              return ComResult::Closed;
            }
            let n = res.unwrap();
            remainder += &String::from_utf8(buf[..n].to_vec()).expect("Intercom cannot parse UTF8");
            for s in remainder.split('\0') {
              //println!("Received Data: {:?}", s);
              match serde_json::from_str::<IntercomMessage>(s) {
                Ok(content) => {
                  println!("Received Intercom: {:?}", content);
                },
                Err(err) => {
                  if let Category::Eof = err.classify() {
                    remainder = s.to_owned();
                    break;
                  } else {
                    panic!("Intercom cannot parse JSON");
                  }
                }
              }
            }
          },
          _ = drop_rx.recv() => {
            println!("Intercom dropped");
            return ComResult::Drop;
          }
        }
    }
}

async fn start_slave(mut state: State<'_, IntercomState>) {
    let addr = ("127.0.0.1:".to_owned() + &state.port.to_string())
        .parse()
        .unwrap();

    loop {
        let socket = TcpSocket::new_v4().expect("Intercom cannot create TcpSocket");
        match socket.connect(addr).await {
          Ok(stream) => {
            if let ComResult::Drop = handle_com(stream, &mut state).await {
              break;
            }
          },
          Err(err) => {
            println!("Slave cannot connect to master, error is {}", err.to_string());
            tokio::time::sleep(Duration::from_millis(1500)).await;
          }
        }

        
    }
}

async fn start_master(mut state: State<'_, IntercomState>) {
    let addr = "127.0.0.1:".to_owned() + &state.port.to_string();

    loop {
        let listener = TcpListener::bind(&addr)
            .await
            .expect("Intercom cannot bind to address");
        println!("Master Intercom accepting connections at {}", addr);
        let (stream, _) = listener
            .accept()
            .await
            .expect("Intercom cannot accept connections");

        if let ComResult::Drop = handle_com(stream, &mut state).await {
            break;
        }
    }
}

fn start(state: State<IntercomState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        if state.master {
            start_master(state).await;
        } else {
            start_slave(state).await;
        }

        Ok(ThreadNext::Terminate)
    })
}

pub fn create(config: &Config, drop_tx: Sender<()>) -> IntercomNode {
    let (send_tx, send_rx) = tokio::sync::broadcast::channel::<IntercomMessage>(64);

    IntercomNode {
        drop_rx: drop_tx.subscribe(),
        send_message_tx: send_tx,
        state: Arc::new(Mutex::new(IntercomState {
            drop_rx: drop_tx.subscribe(),
            send_message_rx: send_rx,
            master: config.intercom.master,
            port: config.intercom.port,
        })),
    }
}

impl Node<IntercomState> for IntercomNode {
    fn get_state_handle(&self) -> Arc<Mutex<IntercomState>> {
        self.state.clone()
    }

    fn get_drop_rx(&self) -> Receiver<()> {
        self.drop_rx.resubscribe()
    }
}

#[async_trait]
impl Executor for IntercomNode {
    async fn init(&self) -> Handles {
        vec![]
    }

    async fn run(&self) -> Handles {
        vec![self.once(start)]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
