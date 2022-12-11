use crate::config::*;
use crate::intercom::{IntercomMessage, IntercomPosition};
use crate::math::Vec2;
use crate::node::*;
use crate::perception::{Measurement, PerceptionMessage};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

struct PerceptionState {
    drop_tx: Sender<()>,
    pos_url: String,
    ball_url: String,
    get_goals_url: String,
    owngoal_url: String,
    targetgoal_url: String,
    boundary_url: String,
    client: reqwest::Client,
    last_ball_position: Option<Vec2>,
    last_owngoal_position: Option<Vec2>,
    last_targetgoal_position: Option<Vec2>,
    last_boundary_position: Option<Vec2>,
    last_abs_robot_pos: Option<Vec2>,
    last_abs_ball_pos: Option<Vec2>,
    last_goals: u32,
    perception_tx: Sender<PerceptionMessage>,
    intercom_send_tx: Sender<IntercomMessage>,
}

pub struct PerceptionNode {
    pub perception_rx: Receiver<PerceptionMessage>,
    drop_rx: Receiver<()>,
    state: Arc<Mutex<PerceptionState>>,
}

#[derive(Deserialize, Debug)]
struct DetectorResponse {
    detected: bool,
    position: Vec2,
}

#[derive(Deserialize, Debug)]
struct PosResponse {
    abs_robot_pos: Vec2,
    abs_ball_pos: Vec2,
}

#[derive(Deserialize, Debug)]
struct GoalResponse {
    n_goals: u32,
}

impl DetectorResponse {
    fn to_option(&self) -> Option<Vec2> {
        if self.detected {
            Some(self.position)
        } else {
            None
        }
    }
}

async fn get_url(
    client: &reqwest::Client,
    url: &String,
    drop_tx: &Sender<()>,
) -> std::result::Result<DetectorResponse, ThreadError> {
    let result = client.get(url).send().await;

    match result {
        Ok(response) => Ok(serde_json::from_str(response.text().await.unwrap().as_str()).unwrap()),
        Err(err) => {
            drop_tx.send(()).unwrap();
            Err(ThreadError::from(err))
        }
    }
}

async fn get_goals(
    client: &reqwest::Client,
    url: &String,
    drop_tx: &Sender<()>,
) -> std::result::Result<GoalResponse, ThreadError> {
    let result = client.get(url).send().await;

    match result {
        Ok(response) => Ok(serde_json::from_str(response.text().await.unwrap().as_str()).unwrap()),
        Err(err) => {
            drop_tx.send(()).unwrap();
            Err(ThreadError::from(err))
        }
    }
}

async fn get_pos(
    client: &reqwest::Client,
    url: &String,
    drop_tx: &Sender<()>,
) -> std::result::Result<PosResponse, ThreadError> {
    let result = client.get(url).send().await;

    match result {
        Ok(response) => Ok(serde_json::from_str(response.text().await.unwrap().as_str()).unwrap()),
        Err(err) => {
            drop_tx.send(()).unwrap();
            Err(ThreadError::from(err))
        }
    }
}

fn query_simulation(mut state: State<PerceptionState>) -> DynFut<NodeResult> {
    Box::pin(async move {
        let res = get_pos(&state.client, &state.pos_url, &state.drop_tx).await?;

        state.last_abs_robot_pos = Some(res.abs_robot_pos);
        state.last_abs_ball_pos = Some(res.abs_ball_pos);

        state.last_goals = get_goals(&state.client, &state.get_goals_url, &state.drop_tx)
            .await?
            .n_goals;

        state.last_ball_position = get_url(&state.client, &state.ball_url, &state.drop_tx)
            .await?
            .to_option();
        state.last_owngoal_position = get_url(&state.client, &state.owngoal_url, &state.drop_tx)
            .await?
            .to_option();

        state.last_targetgoal_position =
            get_url(&state.client, &state.targetgoal_url, &state.drop_tx)
                .await?
                .to_option();

        state.last_boundary_position = get_url(&state.client, &state.boundary_url, &state.drop_tx)
            .await?
            .to_option();

        state
            .perception_tx
            .send(PerceptionMessage {
                abs_robot_pos: state.last_abs_robot_pos.unwrap(),
                abs_ball_pos: state.last_abs_ball_pos.unwrap(),
                ball: Measurement {
                    position: state.last_ball_position,
                },
                own_goal: Measurement {
                    position: state.last_owngoal_position,
                },
                target_goal: Measurement {
                    position: state.last_targetgoal_position,
                },
                boundary: Measurement {
                    position: state.last_boundary_position,
                },
                n_goals: state.last_goals,
            })
            .unwrap();

        state
            .intercom_send_tx
            .send(IntercomMessage::Position(IntercomPosition {
                ball: state.last_ball_position.unwrap(),
                own_goal: state.last_owngoal_position.unwrap(),
                target_goal: state.last_targetgoal_position.unwrap(),
            }))
            .unwrap();

        Ok(ThreadNext::Next)
    })
}

pub fn create(
    config: &Config,
    drop_tx: Sender<()>,
    intercom_send_tx: Sender<IntercomMessage>,
) -> PerceptionNode {
    let (tx, rx) = tokio::sync::broadcast::channel::<PerceptionMessage>(160);

    PerceptionNode {
        drop_rx: drop_tx.subscribe(),
        perception_rx: rx,
        state: Arc::new(Mutex::new(PerceptionState {
            pos_url: config.simulation.url.to_owned() + "/pos",
            intercom_send_tx,
            perception_tx: tx,
            last_abs_robot_pos: None,
            last_abs_ball_pos: None,
            get_goals_url: config.simulation.url.to_owned() + "/goals",
            ball_url: config.simulation.url.to_owned() + "/ball",
            owngoal_url: config.simulation.url.to_owned() + "/owngoal",
            targetgoal_url: config.simulation.url.to_owned() + "/targetgoal",
            boundary_url: config.simulation.url.to_owned() + "/boundary",
            drop_tx,
            client: reqwest::Client::new(),
            last_ball_position: Some(Vec2 { x: 0.0, y: 0.0 }),
            last_targetgoal_position: Some(Vec2 { x: 0.0, y: 0.0 }),
            last_owngoal_position: Some(Vec2 { x: 0.0, y: 0.0 }),
            last_goals: 0,
            last_boundary_position: None,
        })),
    }
}

impl Node<PerceptionState> for PerceptionNode {
    fn get_state_handle(&self) -> Arc<Mutex<PerceptionState>> {
        self.state.clone()
    }

    fn get_drop_rx(&self) -> Receiver<()> {
        self.drop_rx.resubscribe()
    }
}

#[async_trait]
impl Executor for PerceptionNode {
    async fn init(&self) -> Handles {
        vec![]
    }

    async fn run(&self) -> Handles {
        vec![self.interval(Duration::from_millis(50), query_simulation)]
    }

    async fn stop(&self) -> Handles {
        vec![]
    }
}
