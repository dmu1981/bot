use serde::{Serialize, Deserialize};

use crate::behavior::bt::*;
use crate::behavior::MyBlackboard;
use crate::math::Vec2;
use crate::math::BotNet;
use crate::math::abs;
use crate::math::max;
use crate::math::min;
use crate::motioncontroll::MoveCommand;

use std::slice::Iter;
use genetics::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MyPayload {
    botnet: BotNet,
}


pub struct BTBotNet {
    decorators: Vec<BoxedDecorator<MyBlackboard>>,
    botnet: Option<BotNet>,
    start_time: std::time::Instant,
    ball_score: f32,
    ball_distance_start: f32,
    goal_distance_start: f32,
    dot_score: f32,
    max_goal_distance: f32,
    new_round: bool,
    goal_score: f32,
    iteration: u32, 
    generation: u32,
    score_so_far: f32,
    number_runs: u32,
    round_timer: f32,
    score_counter: f32,
    run: u32,
    genepool: GenePool<MyPayload>,
    toack: Option<GenomeAck<MyPayload>>,
    delay_ticker: u32,
    acc_ball_score: f32,
    acc_goal_score: f32,
    acc_dot_score: f32,    
}

impl BTBotNet {
    pub fn new(url: String) -> Box<BTBotNet> {
        let genepool = GenePool::<MyPayload>::new(150, FitnessSortingOrder::LessIsBetter, url).unwrap();

        let bot = Box::new(BTBotNet {
            toack: None,
            decorators: Vec::<BoxedDecorator<MyBlackboard>>::new(),
            botnet: None,
            start_time: std::time::Instant::now(),
            ball_score: 99999.0,
            goal_score: 99999.0,
            dot_score: -1.0,
            ball_distance_start: 99999.0,
            goal_distance_start: 99999.0,
            max_goal_distance: 0.0,
            new_round: true,
            score_so_far: 0.0,
            round_timer: 4.0,
            run: 0,
            iteration: 0,
            score_counter: 0.0,
            number_runs: 0,
            genepool,
            delay_ticker: 0,
            acc_ball_score: 0.0,
            acc_goal_score: 0.0,
            acc_dot_score: 0.0,
            generation: 0,
        });

        bot
    }

    fn start_next(&mut self) {
      self.ball_score = 99999.0;
      self.goal_score = 99999.0;
      self.score_so_far = 0.0;
      self.run = 0;
      self.number_runs = 4;
      self.new_round = true;
      self.ball_distance_start = 99999.0;
      self.goal_distance_start = 99999.0;
      self.max_goal_distance = 0.0;
      self.delay_ticker = 4;
      self.round_timer = 3.0;
      self.score_counter = 0.0;
      self.acc_ball_score = 0.0;
      self.acc_goal_score = 0.0;
      self.acc_dot_score = 0.0;
      self.generation = 0;

      //println!("Starting next...");
      match self.genepool.poll_one() {
        Ok(gene) => {
          self.number_runs = 1 + gene.message.generation / 25;
          if self.number_runs > 6 {
            self.number_runs = 6;
          }
          self.round_timer = 3.0 + (gene.message.generation as f32) / 20.0;
          if self.round_timer > 6.0 {
            self.round_timer = 6.0;
          }
          println!("Bot generation is {}, playing {} rounds, {}s each", gene.message.generation, self.number_runs, self.round_timer);
          self.generation = gene.message.generation;
          self.botnet = Some(gene.message.payload.botnet.clone());
          self.toack = Some(gene);          
        }
        Err(_) => {
          println!("Genepool returned error... waiting");
          self.botnet = None;
          self.toack = None;
        }
      }
      
    }      
}

#[derive(Serialize)]
struct LogData {
  generation: u32,
  ball_score: f32,
  goal_score: f32,
  dot_score: f32,
  score: f32,
}

impl BTNode<MyBlackboard> for BTBotNet {
    fn get_decorators(&self) -> Iter<Box<dyn BTDecorator<MyBlackboard>>> {
        self.decorators.iter()
    }    

    fn internal_tick(&mut self, blackboard: &mut Box<MyBlackboard>) -> BTResult {
        if self.delay_ticker > 0
        {
          self.delay_ticker -= 1;
          if self.delay_ticker == 0 {
            self.ball_distance_start = blackboard.ball.magnitude();
            self.goal_distance_start = (blackboard.ball - blackboard.target_goal).magnitude();
            self.max_goal_distance = self.goal_distance_start;
            self.ball_score = self.ball_distance_start;
            self.goal_score = self.goal_distance_start;
            self.dot_score = -1.0;
            self.start_time = std::time::Instant::now();
            self.score_counter = 0.0;
            //println!("Ball Distance Start is {}", self.ball_distance_start);
            //println!("Goal Distance Start is {}", self.goal_distance_start);
          }
          else {
            return BTResult::Pending;            
          }
        }
        
        if let None = self.botnet {
          self.start_next();
          
          return BTResult::Pending;
        }

        if self.start_time.elapsed().as_secs_f32() > self.round_timer {
          //let score = /*self.ball_score + self.goal_score * 4.0 + */(self.max_goal_distance - 22.0)*80.0;
          //let max_score = max((self.max_goal_distance / self.goal_distance_start).powf(2.0) - 1.0, 0.0);
          let ball_score = 50.0 * max(((self.ball_score / self.score_counter) / self.ball_distance_start - 0.2) / 0.8, 0.0);
          let goal_score = 400.0 * (max((self.goal_score / self.score_counter) / self.goal_distance_start, 0.0)).powf(1.5);
          let dot_score = 50.0 * (1.0 - self.dot_score / self.score_counter) / 2.0;

          let score = max(goal_score + ball_score + dot_score, 0.0);

          //println!("Score this time was: {} ({} + {} + {})",score, goal_score, ball_score, dot_score);
          self.score_so_far += score;
          self.acc_ball_score += ball_score;
          self.acc_goal_score += goal_score;
          self.acc_dot_score  += dot_score; 
          self.run += 1;
          if self.run == self.number_runs {
            self.iteration += 1;
            
            let log = LogData {
              generation: self.generation,
              ball_score: self.acc_ball_score / (self.run as f32),
              goal_score: self.acc_goal_score / (self.run as f32),
              dot_score: self.acc_dot_score / (self.run as f32),
              score: self.score_so_far / (self.run as f32),
            };
            println!("{}", serde_json::to_string(&log).unwrap());
            //println!("Bot {}... score was {}", self.iteration, self.score_so_far / self.run as f32);

            let toack = self.toack.take();
            if let Some(mut ack) = toack {
              ack.message.fitness = Some(self.score_so_far / self.run as f32);
              self.genepool.ack_one(ack).unwrap();
            }

            
            self.start_next();
            blackboard.reset_sim_tx.send(true).unwrap();
            return BTResult::Pending;
          }
          else {
            
          }

          blackboard.reset_sim_tx.send(true).unwrap();
          self.delay_ticker = 4;
          self.start_time = std::time::Instant::now();

        }

        let dot = blackboard.ball.normalize().dot(&blackboard.target_goal.normalize());
        //println!("{}",dot);
        
        /*self.dot_score = max(self.dot_score, dot);
        self.ball_score = min(self.ball_score, blackboard.ball.magnitude());
        self.goal_score = min(self.goal_score, (blackboard.ball - blackboard.target_goal).magnitude());
        self.max_goal_distance = max(self.max_goal_distance, (blackboard.ball - blackboard.target_goal).magnitude());*/
        self.dot_score += dot;
        self.ball_score += blackboard.ball.magnitude();
        self.goal_score += (blackboard.ball - blackboard.target_goal).magnitude();
        self.score_counter += 1.0;
        self.max_goal_distance = max(self.max_goal_distance, (blackboard.ball - blackboard.target_goal).magnitude());

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
          let target_orientation = blackboard.target_goal.normalize();

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
