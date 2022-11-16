use crate::config::*;
use crate::node::*;
use rppal::gpio::{Gpio, OutputPin};
use tokio::sync::broadcast;

pub struct GPIOPinConfig {
    pin1: OutputPin,
    pin2: OutputPin,
    pin3: OutputPin,
    pin4: OutputPin,
}

pub struct WheelControllerState {
    pins: GPIOPinConfig,
    rolling_counter: u8,
    tx: tokio::sync::broadcast::Sender<crate::wheelcontroller::WheelSpeed>,
}

pub type WheelControllerNode = BotNode<WheelControllerState>;

pub fn init(
    mut state: tokio::sync::MutexGuard<'_, WheelControllerState>,
) -> DynFut<'_, NodeResult> {
    Box::pin(async move {
        state.pins.pin1.set_low();
        state.pins.pin2.set_low();
        state.pins.pin3.set_low();
        state.pins.pin4.set_low();

        Ok(ThreadNext::Terminate)
    })
}

pub fn reset_pins(
    mut state: tokio::sync::MutexGuard<'_, WheelControllerState>,
) -> DynFut<'_, NodeResult> {
    Box::pin(async move {
        state.pins.pin1.set_low();
        state.pins.pin2.set_low();
        state.pins.pin3.set_low();
        state.pins.pin4.set_low();

        Ok(ThreadNext::Terminate)
    })
}

pub fn toggle_pins(mut state: tokio::sync::MutexGuard<'_, WheelControllerState>) -> NodeResult {
    match state.rolling_counter {
        0 => {
            state.pins.pin1.set_high();
            state.pins.pin2.set_low();
            state.pins.pin3.set_low();
            state.pins.pin4.set_low();
        }
        1 => {
            state.pins.pin1.set_high();
            state.pins.pin2.set_high();
            state.pins.pin3.set_low();
            state.pins.pin4.set_low();
        }
        2 => {
            state.pins.pin1.set_low();
            state.pins.pin2.set_high();
            state.pins.pin3.set_low();
            state.pins.pin4.set_low();
        }
        3 => {
            state.pins.pin1.set_low();
            state.pins.pin2.set_high();
            state.pins.pin3.set_high();
            state.pins.pin4.set_low();
        }
        4 => {
            state.pins.pin1.set_low();
            state.pins.pin2.set_low();
            state.pins.pin3.set_high();
            state.pins.pin4.set_low();
        }
        5 => {
            state.pins.pin1.set_low();
            state.pins.pin2.set_low();
            state.pins.pin3.set_high();
            state.pins.pin4.set_high();
        }
        6 => {
            state.pins.pin1.set_low();
            state.pins.pin2.set_low();
            state.pins.pin3.set_low();
            state.pins.pin4.set_high();
        }
        7 => {
            state.pins.pin1.set_high();
            state.pins.pin2.set_low();
            state.pins.pin3.set_low();
            state.pins.pin4.set_high();
        }
        _ => {
            panic!("MotorStepper encountered unknown counter!");
        }
    }

    state.rolling_counter = (state.rolling_counter + 1) % 8;

    Ok(ThreadNext::Next)
}

pub async fn wheel_speed_tx(
    wc: &WheelControllerNode,
) -> tokio::sync::broadcast::Sender<crate::wheelcontroller::WheelSpeed> {
    wc.state.lock().await.tx.clone()
}

fn create(
    wheel: &crate::config::Wheel,
    gpio: &Gpio,
    drop_tx: broadcast::Receiver<()>,
) -> WheelControllerNode {
    let (tx, rx) = tokio::sync::broadcast::channel::<crate::wheelcontroller::WheelSpeed>(4);

    WheelControllerNode::new(
        "Wheel Controller (Raspberry)".to_string(),
        drop_tx,
        WheelControllerState {
            pins: GPIOPinConfig {
                pin1: gpio.get(wheel.gpiopins[0]).unwrap().into_output(),
                pin2: gpio.get(wheel.gpiopins[1]).unwrap().into_output(),
                pin3: gpio.get(wheel.gpiopins[2]).unwrap().into_output(),
                pin4: gpio.get(wheel.gpiopins[3]).unwrap().into_output(),
            },
            rolling_counter: 0,
            tx,
        },
    )
}

pub fn create_all(config: &Config, drop_tx: &broadcast::Sender<()>) -> Vec<WheelControllerNode> {
    let gpio = Gpio::new().unwrap();

    let mut controller = Vec::<WheelControllerNode>::new();

    for wheel in config.wheels.iter() {
        controller.push(create(wheel, &gpio, drop_tx.subscribe()));
    }

    controller
}
