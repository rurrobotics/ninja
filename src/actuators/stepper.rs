use embassy_rp::{
    Peri,
    gpio::{Level, Output, Pin},
    pio::{
        self, Common, Config, Direction, Instance, LoadedProgram, PioPin, StateMachine,
        program::pio_asm,
    },
    pio_programs::clock_divider::calculate_pio_clock_divider,
};

use crate::config::STEPPER_DEFAULT_FREQUENCY;

pub const INSTRUCTION_COUNT: u32 = 32 + 32 + 1;

pub struct PioStepperProgram<'a, PIO: Instance> {
    pub prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioStepperProgram<'a, PIO> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_asm!(
            ".wrap_target",
            "pull block",
            "out x, 32",
            "loop:",
            "    set pins, 1  [31]",
            "    set pins, 0  [31]",
            "    jmp x-- loop",
            ".wrap"
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

pub struct Stepper<'d, T: Instance, const SM: usize> {
    // irq: Irq<'d, T, SM>,
    pub sm: StateMachine<'d, T, SM>,
    pub dir: Output<'d>,
    pub stp: pio::Pin<'d, T>,
}

impl<'d, T: Instance, const SM: usize> Stepper<'d, T, SM> {
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        // irq: Irq<'d, T, SM>,
        stp: Peri<'d, impl PioPin>,
        dir: Peri<'d, impl Pin>,
        program: &PioStepperProgram<'d, T>,
    ) -> Self {
        let stp = pio.make_pio_pin(stp);
        let dir = Output::new(dir, Level::Low);
        sm.set_pin_dirs(Direction::Out, &[&stp]);

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&stp]);

        cfg.clock_divider =
            calculate_pio_clock_divider(STEPPER_DEFAULT_FREQUENCY * INSTRUCTION_COUNT);

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self { sm, dir, stp }
    }

    pub fn set_frequency(&mut self, freq: u32) {
        let clock_divider = calculate_pio_clock_divider(freq * INSTRUCTION_COUNT);
        let divider_f32 = clock_divider.to_num::<f32>();
        assert!(divider_f32 <= 65536.0, "clkdiv must be <= 65536");
        assert!(divider_f32 >= 1.0, "clkdiv must be >= 1");

        self.sm.set_clock_divider(clock_divider);
        self.sm.clkdiv_restart();
    }

    pub async fn step(&mut self, steps: i32) {
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
}
