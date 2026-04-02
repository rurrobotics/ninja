use embassy_rp::{
    Peri,
    gpio::{Pin, Pull},
    pio::{Common, Config, Direction, Instance, Irq, PioPin, StateMachine, program::pio_asm},
    pio_programs::clock_divider::calculate_pio_clock_divider,
};

use crate::{
    actuators::{
        NoAcc, PioStepperProgram, Stepper, AccMode
    },
    config::{
        EXTENSION_FREQUENCY, EXTENSION_HOME_FREQUENCY, EXTENSION_HOME_OFFSET,
        EXTENSION_PULL_OFFSET,
    },
};

pub struct Extension<'d, T: Instance, const SM: usize> {
    stepper: Stepper<'d, T, SM, NoAcc>,
    home: embassy_rp::pio::Pin<'d, T>,
    max_steps: i32,
}

impl<'d, T: Instance, const SM: usize> Extension<'d, T, SM> {
    pub fn new(
        pio: &mut Common<'d, T>,
        sm: StateMachine<'d, T, SM>,
        irq: Irq<'d, T, SM>,
        stp: Peri<'d, impl PioPin>,
        dir: Peri<'d, impl Pin>,
        home: Peri<'d, impl PioPin>,
        program: &PioStepperProgram<'d, T, false>,
    ) -> Self {
        let mut home = pio.make_pio_pin(home);
        home.set_pull(Pull::Up);

        let mut stepper = Stepper::<'d, T, SM, NoAcc>::new(pio, sm, irq, stp, dir, program);
        stepper.set_frequency(EXTENSION_FREQUENCY);

        Self {
            stepper,
            home,
            max_steps: 0,
        }
    }

    pub fn set_frequency(&mut self, freq: u32) {
        self.stepper.set_frequency(freq);
    }

    pub async fn push(&mut self) {
        self.stepper.step(self.max_steps).await;
    }

    pub async fn pull(&mut self) {
        self.stepper
            .step(EXTENSION_PULL_OFFSET - self.max_steps)
            .await;
    }
    pub async fn home(
        &mut self,
        pio: &mut Common<'d, T>,
        stepper_prg: &PioStepperProgram<'d, T, false>,
    ) {
        self.stepper.sm.set_enable(false);
        let homing_prg = pio_asm!(
            "    set x, 0",
            "    jmp pin loop",
            "    jmp end",
            "loop:",
            "    set pins, 1 [31]",
            "    set pins, 0 [31]",
            "    jmp x-- decr",
            "decr:",
            "    jmp pin loop",
            "end:",
            "    in x, 32",
            "    push block",
        );
        let homing_prg = pio.load_program(&homing_prg.program);
        let mut cfg = Config::default();
        cfg.set_set_pins(&[&self.stepper.stp]);
        cfg.set_jmp_pin(&self.home);
        cfg.clock_divider = calculate_pio_clock_divider(EXTENSION_HOME_FREQUENCY * 66);
        cfg.use_program(&homing_prg, &[]);
        self.stepper.sm.set_config(&cfg);
        self.stepper
            .sm
            .set_pin_dirs(Direction::Out, &[&self.stepper.stp]);
        self.stepper.sm.set_pin_dirs(Direction::In, &[&self.home]);
        self.stepper.sm.set_enable(true);

        self.max_steps =
            (u32::MAX - self.stepper.sm.rx().wait_pull().await + 1) as i32 - EXTENSION_HOME_OFFSET;

        self.stepper.sm.set_enable(false);
        let mut cfg = Config::default();
        cfg.set_set_pins(&[&self.stepper.stp]);
        cfg.clock_divider = calculate_pio_clock_divider(EXTENSION_FREQUENCY * NoAcc::INSTRUCTION_COUNT);
        cfg.use_program(&stepper_prg.prg, &[]);
        self.stepper
            .sm
            .set_pin_dirs(Direction::Out, &[&self.stepper.stp]);
        self.stepper.sm.set_config(&cfg);
        self.stepper.sm.set_enable(true);

        self.stepper.step(EXTENSION_HOME_OFFSET).await;
    }
}
