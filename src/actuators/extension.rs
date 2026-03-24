use embassy_rp::{
    Peri,
    gpio::{Input, Pin, Pull},
    pio::{Common, Instance, PioPin, StateMachine},
};
use embassy_time::Timer;

use crate::{
    actuators::{PioStepperProgram, stepper::Stepper},
    config::{EXTENSION_HOME_WAIT, EXTENSION_MAX_STEP},
};

pub struct Extension<'d, T: Instance, const SM: usize> {
    stepper: Stepper<'d, T, SM>,
    home: Input<'d>,
}

impl<'d, T: Instance, const SM: usize> Extension<'d, T, SM> {
    pub fn new(
        pio: &mut Common<'d, T>,
        sm: StateMachine<'d, T, SM>,
        // irq: Irq<'d, T, SM>,
        stp: Peri<'d, impl PioPin>,
        dir: Peri<'d, impl Pin>,
        home: Peri<'d, impl Pin>,
        program: &PioStepperProgram<'d, T>,
    ) -> Self {
        let home = Input::new(home, Pull::Up);

        Self {
            stepper: Stepper::new(pio, sm, stp, dir, program),
            home,
        }
    }

    pub async fn push(&mut self) {
        self.stepper.step(EXTENSION_MAX_STEP).await;
    }

    pub async fn pull(&mut self) {
        self.stepper.step(-EXTENSION_MAX_STEP).await;
    }

    pub async fn home(&mut self) {
        // TODO: Fix this
        while self.home.is_high() {
            self.stepper.step(-1).await;
            Timer::after_millis(EXTENSION_HOME_WAIT).await;
        }
        self.stepper.step(1).await;
    }
}
