# Bot Controller
## On this repository
This repository is part of a bigger project where a four-wheel robot is to be controlled such that it pushes a ball into a given goal. Eventually, this controller will run on an embedded device to control an actual robot but for now, this code provides the basic infrastructure to develop the behavior. It can connect to a simulation written in Unity Engine (not part of this repository) and exchange perception (position of the ball and goals) as well as actuations (wheel rotations) with the simulation in a closed-loop fashion. 

A neural network is used to control the behavior of the bot. The network is provided with the bots perception and provides target position and rotation of the bot. 

A genetic algorithm is used to learn the weights in the neural network. 

## SW Architecture
The bot controller is written in [RUST](https://rust-lang.org) using the famous [TOKIO](https://tokio.rs) library for asynchronous execution (async/await). 

Individual nodes run in seperate TOKIO tasks and communicate via dedicated broadcast channels with each other. The basic architecture of the bot controller looks like the following

### Manager
A dedicated [manager node](https://github.com/dmu1981/bot/blob/main/src/manager/main.rs) is used control communication with the simulation.

### Intercom
The [intercom node](https://github.com/dmu1981/bot/blob/main/src/intercom/main.rs) will handle communication between multiple bots (e.g. to exchange their respective position via Bluetooth)

### Wheel Controller
The [wheel controller node](https://github.com/dmu1981/bot/blob/main/src/wheelcontroller/wc_sim.rs) is responsible to provide information about the extrinsic parameters of the wheels (currently those are pulled from the simulation) and to control the wheel motors (or communicate with the simulation).

### Motion Controlller
The [motion controller](https://github.com/dmu1981/bot/blob/main/src/motioncontroll/motioncontroller.rs) receives a target position and orientation and determines the target angular velocity for each wheel to assume those. 

### Perception
The [perception node](https://github.com/dmu1981/bot/blob/main/src/perception/simulation.rs) will eventually handle detection and tracking of the ball and goals in the real world environment. For now, it connects to the simulation to pull this information directly from there.

### Kicker
The [kicker](https://github.com/dmu1981/bot/blob/main/src/kicker/main.rs) module will eventually controll a dedicated kicker hardware of the bot. For now, it forwards a kick request to the simulation for execution. 

### Behavior
A simple [behavior tree](https://github.com/dmu1981/bot/blob/main/src/behavior/main.rs) is implemented to allow implementation of complex bot behavior. For now, only a single node in the behavior tree is used to run the neural network. More behavior (e.g. avoid the white barrier lines) will be added later. 

## Neural Network
The [neural network](https://github.com/dmu1981/bot/blob/main/src/behavior/botnet_behavior.rs) is a simple feed-forward network with seven input neurons taking X and Y coordinates of the ball and the target goal as well as the respective distance and the dot-product between these two vectors. It uses 50 hidden neurons and four output neurons representing X and Y of the target position and orientation respectively. It uses the hyperbolic tangens as its activation function on all but the input neurons. 

## Genetic Algorithm
The neural network is trained using a genetic algorithm. Each neural network is simulated closed-loop and the overall fitness of the robot is measured. The fitness function is a weighted average of three components, namely

* the average distance between the robot and the ball
* the average distance between the ball and the goal
* the average (normalized) dot-product between ball and goal vector

Whereas it is obvious that the first two terms need to be minimized, the third term represents the alignment between robot and ball. If the bot manages to position itself "behind" the ball (from the goals perspective), the dot product will be close to 1. Thus the third term needs to be maximed. 

For massiv parallel execution of the genetic algorithm in AWS, the controller communicates with a RabbitMQ message broker. The broker dispatches yet to be tested genomes (network configurations) to workers and receives resulting fitness values in a seperate queue. A dedicated breeder worker is then used to monitor this second queue and, once all genomes are evaluated, breed the next generation of genomes by replicating the best (fittest) genomes.
