use rand_distr::num_traits::pow;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;

use crate::behavior::bt::*;
use crate::behavior::MyBlackboard;
use crate::math::Vec2;
use crate::math::BotNet;
use crate::math::abs;
use crate::math::clamp;
use crate::math::max;
use crate::math::min;
use crate::motioncontroll::MoveCommand;
use uuid::Uuid;
use std::fs::OpenOptions;
use std::io::prelude::*;


use std::slice::Iter;
use genetics::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MyPayload {
    botnet: BotNet,
    experiment: uuid::Uuid,
}

#[derive(Debug, Clone)]
struct MyMessage {
  botnet: BotNet,
  generation: u32,
  experiment: uuid::Uuid,
  fitness: f32,
}


pub struct BTBotNetWatcher {
    decorators: Vec<BoxedDecorator<MyBlackboard>>,
    botnet: Option<BotNet>,
    n_goals: u32,
    start_time: std::time::Instant,    
    genepool: GenePool<MyPayload>,
    generation: u32,
    experiment: Option<Uuid>,
    genes: Vec<MyMessage>,
    rx: Receiver<Vec<MyMessage>>,
    rx_gen: Receiver<u32>,
    watcher_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GenerationResponse {
  generation: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WatcherState {
  experiment: String,
  generation: u32,
  maxGeneration: u32,
  score: f32,
}

async fn poll_generation(tx: Sender<u32>, url: String) {
  let client = reqwest::Client::new();
  let mut was_generation: Option<u32> = None;
  loop {
    let result = client.get(&url).send().await;

    match result {
      Ok(response) => {
        let r : GenerationResponse = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
        if let Some(was) = was_generation {
          if was != r.generation {
            was_generation = Some(r.generation);
            tx.send(r.generation).unwrap();
          }        }
        else {
          was_generation = Some(r.generation);
          tx.send(r.generation).unwrap();
        } 
      },
      Err(err) => {

      }
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
  }
}

async fn poll_best(tx: Sender<Vec<MyMessage>>, url: String) 
{
  let mut genepool = GenePool::<MyPayload>::new(250, FitnessSortingOrder::LessIsBetter, url).unwrap();
  loop {
    println!("Requesting next bot");
    match genepool.poll_best() {
      Ok(mut res) => {
        res.sort_by(|a, b| a.generation.cmp(&b.generation));
        res.reverse();

        let MessageVec = res.into_iter().map(|gene| {
          MyMessage {
            botnet: gene.payload.botnet, 
            generation: gene.generation, 
            experiment: gene.payload.experiment,
            fitness: gene.fitness.unwrap(),
          }
        }).collect();

        //let gene = res.into_iter().nth(50).unwrap();
        //println!("Next bot generation is {}, Score was {}", gene.generation, gene.fitness.unwrap());
        tx.send(MessageVec).unwrap();        
      },
      Err(_) => {

      }
    }

    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
  }
}

impl BTBotNetWatcher {
    pub fn new(pool_url: String, sim_url: String) -> Box<BTBotNetWatcher> {
        let (tx, rx) = tokio::sync::broadcast::channel(4);
        let (tx_gen, rx_gen) = tokio::sync::broadcast::channel(4);

        let url_clone = pool_url.clone();

        tokio::spawn(async move {
          poll_best(tx, url_clone).await;
        });

        let sim_url_clone = sim_url.clone();
        tokio::spawn(async move {
          poll_generation(tx_gen, sim_url_clone + "/generation").await;
        });

        let genepool = GenePool::<MyPayload>::new(250, FitnessSortingOrder::LessIsBetter, pool_url).unwrap();

        

        let bot = Box::new(BTBotNetWatcher {
            watcher_url: sim_url.clone() + "/watcher",
            decorators: Vec::<BoxedDecorator<MyBlackboard>>::new(),
            botnet: None,
            genepool,
            n_goals: 0,
            generation: 0,
            start_time: std::time::Instant::now(),
            experiment: None,
            rx,
            rx_gen,
            genes: Vec::<MyMessage>::new(),
        });

        bot
    }

    fn start_next(&mut self) -> bool {
      match self.rx_gen.try_recv() {
        Ok(gen) => {
          let client = reqwest::Client::new();

          for gene in &self.genes {
            if gene.generation == gen {
              println!("Current bot is generation {}", gen);
              self.botnet = Some(gene.botnet.clone());
              self.experiment = Some(gene.experiment);
              self.generation = gene.generation;
              self.n_goals = 0;
              self.start_time = std::time::Instant::now();

              let watcherState = WatcherState {
                experiment: gene.experiment.to_string(),
                generation: gene.generation,
                maxGeneration: self.genes.len() as u32,
                score: gene.fitness,
              };

              let url = self.watcher_url.clone();
              tokio::spawn(async move {
                client.post(&url).body(serde_json::to_string(&watcherState).unwrap()).send().await.unwrap();
              });

              return true;
            }
          }          
        },
        Err(_) => {

        }
      }

      match self.rx.try_recv() {
        Ok(msg) => {
          let client = reqwest::Client::new();

          let mut auto_step = false;
          if self.genes.len() > 0 {
            let max_generation = self.genes.iter().reduce(|accum, item| {
              if accum.generation > item.generation { accum } else { item }
            }).unwrap().generation;
            if self.generation == max_generation {
              auto_step = true;
            }
          }
          self.genes = msg;
          if auto_step {
            self.generation = self.genes.len() as u32;
          }

          for gene in &self.genes {
            if gene.generation == self.generation {
              println!("---Current bot is generation {}", self.generation);

              let watcherState = WatcherState {
                experiment: gene.experiment.to_string(),
                generation: gene.generation,
                maxGeneration: self.genes.len() as u32,
                score: gene.fitness,
              };

              let url = self.watcher_url.clone();
              tokio::spawn(async move {
                client.post(&url).body(serde_json::to_string(&watcherState).unwrap()).send().await.unwrap();
              });

              self.n_goals = 0;
              self.botnet = Some(gene.botnet.clone());
              self.experiment = Some(gene.experiment);
              self.generation = gene.generation;
              self.n_goals = 0;
              self.start_time = std::time::Instant::now();

              break;
            }
          }   

          
          return true;
        },
        Err(_) => {

        }
      }

      return false;
    }      
    
    
}



impl BTNode<MyBlackboard> for BTBotNetWatcher {
    fn get_decorators(&self) -> Iter<Box<dyn BTDecorator<MyBlackboard>>> {
        self.decorators.iter()
    }    


    fn internal_tick(&mut self, blackboard: &mut Box<MyBlackboard>) -> BTResult {

        
        if self.start_next() == true {
          self.start_time = std::time::Instant::now();
          blackboard.reset_sim_tx.send(true).unwrap();
          return BTResult::Pending;
        }

        if blackboard.n_goals > self.n_goals {
          println!("GOAL!");
          self.n_goals = blackboard.n_goals;
          self.start_time = std::time::Instant::now();
        }

        if self.start_time.elapsed().as_secs_f32() > 8.0 {
          self.start_next();
          self.start_time = std::time::Instant::now();
          blackboard.reset_sim_tx.send(true).unwrap();
          return BTResult::Pending;
        }

        let dot = blackboard.ball.normalize().dot(&blackboard.target_goal.normalize());
        
        //println!("x: {}, y: {}", blackboard.ball.x, blackboard.ball.y);
        let input = vec![
          blackboard.ball.x, 
          blackboard.ball.y, 
          blackboard.target_goal.x, 
          blackboard.target_goal.y,
          blackboard.ball.magnitude(),
          blackboard.target_goal.magnitude(),
          dot];

        if let Some(net) = &self.botnet {
          let output = net.fwd(input);

          let target_position = Vec2 { x: output[0] * 10.0, y: output[1] * 10.0 };
          let orientation = Vec2 { x: output[2], y: output[3] }.normalize();

          let steps : u32;
          if self.generation > 9 {
            steps = (self.generation - 10) / 10;
          } else {
            steps = 0;
          }

          let r = clamp((steps as f32) * 0.05, 0.05, 0.3);
          let target_orientation = blackboard.target_goal.normalize().lerp(&orientation, r);

          blackboard
              .movecommand_tx
              .send(MoveCommand::MoveAndAlign(
                  target_position,
                  target_orientation,
              ))
              .unwrap();

          if abs(blackboard.ball.x) < 0.2
          && abs(blackboard.ball.y - 1.2) < 0.2
              && blackboard.target_goal.magnitude() < 15.0
          {
              blackboard.kicker_tx.send(true).unwrap();
          }
        }
        
        BTResult::Pending
    }
}


