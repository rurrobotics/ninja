use core::f64::consts::PI;

use embassy_futures::join::join;
use embassy_rp::{
    Peri,
    gpio::Pin,
    pio::{Common, Instance, Irq, PioPin, StateMachine},
};

use crate::{
    actuators::{PioStepperProgram, stepper::Stepper},
    config::{
        DRIVETRAIN_FREQUENCY, DRIVETRAIN_STEPS_PER_REVOLUTION, DRIVETRAIN_WHEEL_DIAMETER,
        DRIVETRAIN_WHEEL_DISTANCE,
    },
};

pub struct Drivetrain<'d, T: Instance, const SM1: usize, const SM2: usize> {
    stepper1: Stepper<'d, T, SM1>,
    stepper2: Stepper<'d, T, SM2>,
}

impl<'d, T: Instance, const SM1: usize, const SM2: usize> Drivetrain<'d, T, SM1, SM2> {
    pub fn new(
        pio: &mut Common<'d, T>,
        sm1: StateMachine<'d, T, SM1>,
        irq1: Irq<'d, T, SM1>,
        stp1: Peri<'d, impl PioPin>,
        dir1: Peri<'d, impl Pin>,
        sm2: StateMachine<'d, T, SM2>,
        irq2: Irq<'d, T, SM2>,
        stp2: Peri<'d, impl PioPin>,
        dir2: Peri<'d, impl Pin>,
        program: &PioStepperProgram<'d, T>,
    ) -> Self {
        let mut stepper1 = Stepper::new(pio, sm1, irq1, stp1, dir1, program);
        stepper1.set_frequency(DRIVETRAIN_FREQUENCY);

        let mut stepper2 = Stepper::new(pio, sm2, irq2, stp2, dir2, program);
        stepper2.set_frequency(DRIVETRAIN_FREQUENCY);

        Self { stepper1, stepper2 }
    }

    pub async fn step(&mut self, steps1: i32, steps2: i32) {
        join(self.stepper1.step(steps1), self.stepper2.step(steps2)).await;
        join(self.stepper1.wait(), self.stepper2.wait()).await;
    }

    pub async fn drive(&mut self, distance: f64) -> i32 {
        let steps = (distance * DRIVETRAIN_STEPS_PER_REVOLUTION as f64
            / (DRIVETRAIN_WHEEL_DIAMETER * PI)) as i32;
        self.step(steps, -steps).await;
        steps
    }

    pub async fn turn(&mut self, degrees: f64) -> i32 {
        let distance = degrees * PI / 360.0 * DRIVETRAIN_WHEEL_DISTANCE;
        let steps = (distance * DRIVETRAIN_STEPS_PER_REVOLUTION as f64
            / (DRIVETRAIN_WHEEL_DIAMETER * PI)) as i32;
        self.step(steps, steps).await;
        steps
    }
}
