
//use futures::future::join_all;

mod node;
mod motorstepper;
mod motorcontroller;

use crate::node::Node;
use crate::motorstepper::MotorStepper;
use crate::motorcontroller::MotorController;

use rppal::gpio::Gpio;

type NodeBox = Box<dyn Node + Send>;
type NodeList = Vec<NodeBox>;

fn init_all_nodes(mut node_list : NodeList) -> Vec<tokio::task::JoinHandle<NodeBox>>
{
    let mut handles : Vec<tokio::task::JoinHandle<NodeBox>> = Vec::new();

    while !node_list.is_empty() {
        let mut node = node_list.swap_remove(0);
        handles.push(tokio::spawn(async move {
            node.init().await;

            node
        }));
    }
    
    handles
}

fn run_all_nodes(mut node_list : NodeList) -> Vec<tokio::task::JoinHandle<NodeBox>>
{
    let mut handles : Vec<tokio::task::JoinHandle<NodeBox>> = Vec::new();

    while !node_list.is_empty() {
        let mut node = node_list.swap_remove(0);
        handles.push(tokio::spawn(async move {
            node.run().await;

            node
        }));
    }
    
    handles
}



#[tokio::main]
async fn main() {
    let gpio = Gpio::new().unwrap();

    // Create all nodes
    let stepper = MotorStepper::new(&gpio, 1,2,3,4);
    let controller = MotorController::new(&stepper);

    // Push them into a list
    let mut node_list = NodeList::new();
    node_list.push(stepper);
    node_list.push(controller);

    let handles = init_all_nodes(node_list);
    let threads : NodeList = futures::future::join_all(handles).await.into_iter().map(|x| { x.unwrap() }).collect();

    let handles2 = run_all_nodes(threads);

    futures::future::join_all(handles2).await;
}


