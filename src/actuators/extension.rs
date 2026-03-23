use embassy_rp::{
    Peri,
    gpio::{Input, Level, Output, Pin, Pull},
    pio::{Common, Config, Direction, Instance, PioPin, StateMachine},
    pio_programs::clock_divider::calculate_pio_clock_divider,
};
use embassy_time::Timer;

use crate::{
    actuators::{INSTRUCTION_COUNT, PioStepperProgram},
    config::{EXTENSION_HOME_WAIT, EXTENSION_MAX_STEP, STEPPER_DEFAULT_FREQUENCY},
};

pub struct Extension<'d, T: Instance, const SM: usize> {
    // irq: Irq<'d, T, SM>,
    sm: StateMachine<'d, T, SM>,
    dir: Output<'d>,
    home: Input<'d>,
}

impl<'d, T: Instance, const SM: usize> Extension<'d, T, SM> {
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        // irq: Irq<'d, T, SM>,
        stp: Peri<'d, impl PioPin>,
        dir: Peri<'d, impl Pin>,
        home: Peri<'d, impl Pin>,
        program: &PioStepperProgram<'d, T>,
    ) -> Self {
        let dir = Output::new(dir, Level::Low);
        let home = Input::new(home, Pull::Up);
        let stp = pio.make_pio_pin(stp);
        sm.set_pin_dirs(Direction::Out, &[&stp]);

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&stp]);

        cfg.clock_divider =
            calculate_pio_clock_divider(STEPPER_DEFAULT_FREQUENCY * INSTRUCTION_COUNT);

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);

        Self { sm, dir, home }
    }

    pub fn set_frequency(&mut self, freq: u32) {
        let clock_divider = calculate_pio_clock_divider(freq * INSTRUCTION_COUNT);
        let divider_f32 = clock_divider.to_num::<f32>();
        assert!(divider_f32 <= 65536.0, "clkdiv must be <= 65536");
        assert!(divider_f32 >= 1.0, "clkdiv must be >= 1");

        self.sm.set_clock_divider(clock_divider);
        self.sm.clkdiv_restart();
    }

    async fn step(&mut self, steps: i32) {
        let steps = match steps >= 0 {
            true => {
                self.dir.set_high();
                steps
            }
            false => {
                self.dir.set_low();
                -steps
            }
        };

        self.sm.tx().wait_push(steps as u32).await;
    }

    pub async fn wait(&mut self) {
        // TODO: Fix this
        while !self.sm.tx().empty() {
            embassy_time::Timer::after_micros(10).await;
        }
    }

    pub async fn forward(&mut self) {
        self.step(EXTENSION_MAX_STEP).await;
    }

    pub async fn home(&mut self) {
        while self.home.is_high() {
            self.step(-1).await;
            Timer::after_millis(EXTENSION_HOME_WAIT).await;
        }
    }
}
