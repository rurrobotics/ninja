use core::mem::{self, MaybeUninit};
use embassy_rp::{
    Peri,
    gpio::Output,
    pio::{
        Common, Config, Direction, Instance, Irq, LoadedProgram, PioPin, StateMachine,
        program::pio_asm,
    },
    pio_programs::clock_divider::calculate_pio_clock_divider,
};

pub struct PioStepperProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioStepperProgram<'a, PIO> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_asm!(
            "pull block",
            "mov x, osr",
            "jmp !x end",
            "loop:",
            "    set pins, 1 [31]",
            "    set pins, 0 [31]",
            "    jmp x-- loop",
            "end:",
            // "    irq 0 rel"
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

pub struct PioStepper<'d, T: Instance, const SM: usize> {
    // irq: Irq<'d, T, SM>,
    sm: StateMachine<'d, T, SM>,
    dir: Output<'d>,
}

impl<'d, T: Instance, const SM: usize> PioStepper<'d, T, SM> {
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        // irq: Irq<'d, T, SM>,
        stp: Peri<'d, impl PioPin>,
        dir: Output<'d>,
        program: &PioStepperProgram<'d, T>,
    ) -> Self {
        let stp = pio.make_pio_pin(stp);
        sm.set_pin_dirs(Direction::Out, &[&stp]);

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&stp]);

        // TODO: Check this value
        cfg.clock_divider = calculate_pio_clock_divider(1000);

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);

        Self { sm, dir }
    }

    pub fn set_frequency(&mut self, freq: u32) {
        // TODO: Magic value copied from 4 pin stepper, inspect this
        let clock_divider = calculate_pio_clock_divider(freq * 136);
        let divider_f32 = clock_divider.to_num::<f32>();
        assert!(divider_f32 <= 65536.0, "clkdiv must be <= 65536");
        assert!(divider_f32 >= 1.0, "clkdiv must be >= 1");

        self.sm.set_clock_divider(clock_divider);
        self.sm.clkdiv_restart();
    }

    pub async fn step(&mut self, steps: i32) {
        let steps = match steps >= 0 {
            true => {
                self.dir.is_set_high();
                steps
            }
            false => {
                self.dir.is_set_low();
                -steps
            }
        };

        self.sm.tx().wait_push(steps as u32).await;
    }

    pub async fn run() {
        
    }

    pub async fn wait(&mut self) {
        while !self.sm.tx().empty() {
            embassy_time::Timer::after_micros(10).await;
        }
    }
}
