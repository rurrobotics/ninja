use core::mem::{self, MaybeUninit};

use crate::config::STEPPER_WITHACC_TIMER_FREQUENCY;
use embassy_rp::{
    Peri,
    dma::Channel,
    gpio::{Level, Output, Pin},
    pio::{
        self, Common, Config, Direction, Instance, Irq, LoadedProgram, PioPin, StateMachine,
        program::pio_asm,
    },
    pio_programs::clock_divider::calculate_pio_clock_divider,
};

pub struct PioStepperProgram<'a, PIO: Instance, const ACC: bool> {
    pub prg: LoadedProgram<'a, PIO>,
}

pub trait AccMode {
    type Dma<'d>;
    const INSTRUCTION_COUNT: u32;
}

pub struct NoAcc;
pub struct WithAcc<C: Channel>(core::marker::PhantomData<C>);

impl AccMode for NoAcc {
    type Dma<'d> = ();
    const INSTRUCTION_COUNT: u32 = 32 + 32 + 1;
}

impl<C: Channel> AccMode for WithAcc<C> {
    type Dma<'d> = Peri<'d, C>;
    const INSTRUCTION_COUNT: u32 = 32 + 32 + 4;
}

impl<'a, PIO: Instance> PioStepperProgram<'a, PIO, false> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_asm!(
            ".wrap_target",
            "pull block",
            "out x, 32",
            "loop:",
            "    set pins, 1  [31]",
            "    set pins, 0  [31]",
            "    jmp x-- loop",
            "    irq 0 rel",
            ".wrap"
        );
        let prg = common.load_program(&prg.program);
        Self { prg }
    }
}

impl<'a, PIO: Instance> PioStepperProgram<'a, PIO, true> {
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_asm!(
            ".wrap_target",
            "pull block",
            "out x, 32",
            "loop:",
            "    pull block",
            "    out y, 32",
            "    set pins, 1  [31]",
            "    set pins, 0  [31]",
            "delay:",
            "    jmp y-- delay"
            "    jmp x-- loop",
            "    irq 0 rel",
            ".wrap"
        );
        let prg = common.load_program(&prg.program);
        Self { prg }
    }
}

pub struct Stepper<'d, T: Instance, const SM: usize, A: AccMode> {
    irq: Irq<'d, T, SM>,
    pub sm: StateMachine<'d, T, SM>,
    pub dir: Output<'d>,
    pub stp: pio::Pin<'d, T>,
    pub dma: A::Dma<'d>,
}

impl<'d, T: Instance, const SM: usize, A: AccMode> Stepper<'d, T, SM, A> {
    #[inline]
    pub fn convert_steps(steps: i32) -> Option<u32> {
        match steps {
            1.. => Some(steps as u32 - 1),
            ..=-1 => Some((-steps) as u32 - 1),
            0 => None,
        }
    }

    #[inline]
    pub fn update_direction(&mut self, steps: i32) {
        match steps {
            1.. => {
                self.dir.set_high();
            }
            ..=-1 => {
                self.dir.set_low();
            }
            0 => {}
        }
    }
}

impl<'d, T: Instance, const SM: usize> Stepper<'d, T, SM, NoAcc> {
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        irq: Irq<'d, T, SM>,
        stp: Peri<'d, impl PioPin>,
        dir: Peri<'d, impl Pin>,
        program: &PioStepperProgram<'d, T, false>,
        frequency: u32,
    ) -> Self {
        let stp = pio.make_pio_pin(stp);
        let dir = Output::new(dir, Level::Low);
        sm.set_pin_dirs(Direction::Out, &[&stp]);

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&stp]);

        cfg.clock_divider = calculate_pio_clock_divider(frequency * NoAcc::INSTRUCTION_COUNT);

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self {
            irq,
            sm,
            dir,
            stp,
            dma: (),
        }
    }

    pub fn set_frequency(&mut self, freq: u32) {
        let clock_divider = calculate_pio_clock_divider(freq * NoAcc::INSTRUCTION_COUNT);
        let divider_f32 = clock_divider.to_num::<f32>();
        assert!(divider_f32 <= 65536.0, "clkdiv must be <= 65536");
        assert!(divider_f32 >= 1.0, "clkdiv must be >= 1");

        self.sm.set_clock_divider(clock_divider);
        self.sm.clkdiv_restart();
    }

    pub async fn step(&mut self, steps: i32) {
        self.update_direction(steps);
        let Some(steps) = Self::convert_steps(steps) else {
            return;
        };
        self.sm.tx().wait_push(steps).await;
        let drop = OnDrop::new(|| {
            self.sm.clear_fifos();
            unsafe {
                self.sm.exec_jmp(0);
            }
        });
        self.irq.wait().await;
        drop.defuse();
    }
}

impl<'d, T: Instance, const SM: usize, C: Channel> Stepper<'d, T, SM, WithAcc<C>> {
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        irq: Irq<'d, T, SM>,
        stp: Peri<'d, impl PioPin>,
        dir: Peri<'d, impl Pin>,
        dma: Peri<'d, C>,
        program: &PioStepperProgram<'d, T, true>,
    ) -> Self {
        let stp = pio.make_pio_pin(stp);
        let dir = Output::new(dir, Level::Low);
        sm.set_pin_dirs(Direction::Out, &[&stp]);

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&stp]);

        cfg.clock_divider = calculate_pio_clock_divider(STEPPER_WITHACC_TIMER_FREQUENCY);

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self {
            irq,
            sm,
            dir,
            stp,
            dma,
        }
    }

    pub async fn step(&mut self, delays: &[u32]) {
        self.sm.tx().wait_push(delays.len() as u32).await;

        self.sm
            .tx()
            .dma_push(self.dma.reborrow(), delays, false)
            .await;

        let drop = OnDrop::new(|| {
            self.sm.clear_fifos();
            unsafe {
                self.sm.exec_jmp(0);
            }
        });
        self.irq.wait().await;
        drop.defuse();
    }
}

struct OnDrop<F: FnOnce()> {
    f: MaybeUninit<F>,
}
impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self {
            f: MaybeUninit::new(f),
        }
    }
    pub fn defuse(self) {
        mem::forget(self)
    }
}
impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        unsafe { self.f.as_ptr().read()() }
    }
}
