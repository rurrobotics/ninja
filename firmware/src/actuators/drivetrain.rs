use core::f64::consts::PI;

use embassy_futures::join::join;
use embassy_rp::{
    Peri,
    dma::Channel,
    gpio::Pin,
    pio::{Common, Instance, Irq, PioPin, StateMachine},
};

use crate::{
    actuators::{
        PioStepperProgram,
        stepper::{Stepper, WithAcc},
    },
    config::{
        DRIVETRAIN_STEPS_PER_REVOLUTION, DRIVETRAIN_WHEEL_DIAMETER, DRIVETRAIN_WHEEL_DISTANCE,
    },
    packet::Color,
    profiles::MotionProfile,
};

pub struct Drivetrain<
    'd,
    T: Instance,
    const SM1: usize,
    const SM2: usize,
    MP: MotionProfile,
    C1: Channel,
    C2: Channel,
> {
    stepper1: Stepper<'d, T, SM1, WithAcc<C1>>,
    stepper2: Stepper<'d, T, SM2, WithAcc<C2>>,
    pub profile: MP,
    color: Color,
}

impl<
    'd,
    T: Instance,
    const SM1: usize,
    const SM2: usize,
    C1: Channel,
    C2: Channel,
    MP: MotionProfile + Copy,
> Drivetrain<'d, T, SM1, SM2, MP, C1, C2>
{
    pub fn new(
        pio: &mut Common<'d, T>,
        sm1: StateMachine<'d, T, SM1>,
        irq1: Irq<'d, T, SM1>,
        stp1: Peri<'d, impl PioPin>,
        dir1: Peri<'d, impl Pin>,
        dma1: Peri<'d, C1>,
        sm2: StateMachine<'d, T, SM2>,
        irq2: Irq<'d, T, SM2>,
        stp2: Peri<'d, impl PioPin>,
        dir2: Peri<'d, impl Pin>,
        dma2: Peri<'d, C2>,
        profile: MP,
        program: &PioStepperProgram<'d, T, true>,
    ) -> Self {
        let stepper1 =
            Stepper::<'d, T, SM1, WithAcc<C1>>::new(pio, sm1, irq1, stp1, dir1, dma1, program);

        let stepper2 =
            Stepper::<'d, T, SM2, WithAcc<C2>>::new(pio, sm2, irq2, stp2, dir2, dma2, program);

        Self {
            stepper1,
            stepper2,
            color: Color::Blue,
            profile,
        }
    }

    pub async fn step(&mut self, steps: i32, inverse: bool) {
        self.stepper1.update_direction(steps);
        self.stepper2.update_direction(match inverse {
            true => -steps,
            false => steps,
        });
        let Some(steps) = Stepper::<'d, T, SM1, WithAcc<C1>>::convert_steps(steps) else {
            return;
        };

        let delays = self.profile.delays(steps);

        join(self.stepper1.step(&delays), self.stepper2.step(&delays)).await;
    }

    #[inline]
    fn calculate_steps(distance: f64) -> i32 {
        (distance * DRIVETRAIN_STEPS_PER_REVOLUTION as f64 / (DRIVETRAIN_WHEEL_DIAMETER * PI))
            as i32
    }

    pub async fn drive(&mut self, distance: f64) -> i32 {
        let steps = Self::calculate_steps(distance);
        self.step(steps, true).await;
        steps
    }

    pub async fn turn(&mut self, degrees: f64) -> i32 {
        let degrees = match self.color {
            Color::Yellow => -degrees,
            Color::Blue => degrees,
        };

        let distance = degrees * PI / 360.0 * DRIVETRAIN_WHEEL_DISTANCE;
        let steps = Self::calculate_steps(distance);
        self.step(steps, false).await;
        steps
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}
