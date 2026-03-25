use core::mem::{self, MaybeUninit};

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

use crate::config::{
    STEPPER_DEFAULT_ACCELERATION, STEPPER_DEFAULT_FREQUENCY, STEPPER_DEFAULT_START_DELAY,
    STEPPER_MAX_ACCELERATION_STEPS,
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
    pub start_delay: u32,
    pub acceleration: u32,
}

impl<'d, T: Instance, const SM: usize, A: AccMode> Stepper<'d, T, SM, A> {
    pub fn set_frequency(&mut self, freq: u32) {
        let clock_divider = calculate_pio_clock_divider(freq * A::INSTRUCTION_COUNT);
        let divider_f32 = clock_divider.to_num::<f32>();
        assert!(divider_f32 <= 65536.0, "clkdiv must be <= 65536");
        assert!(divider_f32 >= 1.0, "clkdiv must be >= 1");

        self.sm.set_clock_divider(clock_divider);
        self.sm.clkdiv_restart();
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
    ) -> Self {
        let stp = pio.make_pio_pin(stp);
        let dir = Output::new(dir, Level::Low);
        sm.set_pin_dirs(Direction::Out, &[&stp]);

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&stp]);

        cfg.clock_divider =
            calculate_pio_clock_divider(STEPPER_DEFAULT_FREQUENCY * NoAcc::INSTRUCTION_COUNT);

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self {
            irq,
            sm,
            dir,
            stp,
            dma: (),
            acceleration: STEPPER_DEFAULT_ACCELERATION,
            start_delay: STEPPER_DEFAULT_START_DELAY,
        }
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

        cfg.clock_divider = calculate_pio_clock_divider(
            STEPPER_DEFAULT_FREQUENCY * <WithAcc<C> as AccMode>::INSTRUCTION_COUNT,
        );

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self {
            irq,
            sm,
            dir,
            stp,
            dma,
            acceleration: STEPPER_DEFAULT_ACCELERATION,
            start_delay: STEPPER_DEFAULT_START_DELAY,
        }
    }

    pub async fn step(&mut self, steps: i32) {
        let steps = match steps >= 0 {
            true => {
                self.dir.set_high();
                steps as u32
            }
            false => {
                self.dir.set_low();
                (-steps) as u32
            }
        };

        let total_delays = steps as usize + 1;
        let full_ramp_len = (self.start_delay / self.acceleration) as usize;
        let ramp_len = full_ramp_len.min(total_delays / 2);
        let cruise_steps = total_delays - 2 * ramp_len;

        let mut accel = heapless::Vec::<u32, STEPPER_MAX_ACCELERATION_STEPS>::new();
        let mut decel = heapless::Vec::<u32, STEPPER_MAX_ACCELERATION_STEPS>::new();
        for i in 0..ramp_len {
            let _ = accel.push(self.start_delay - (i as u32 * self.acceleration));
            let _ = decel.push(self.start_delay - ((ramp_len - 1 - i) as u32 * self.acceleration));
        }

        self.sm.tx().wait_push(steps).await;

        if !accel.is_empty() {
            self.sm
                .tx()
                .dma_push(self.dma.reborrow(), &accel, false)
                .await;
        }

        if cruise_steps != 0 {
            self.sm
                .tx()
                .dma_push_repeated::<_, u32>(self.dma.reborrow(), cruise_steps)
                .await;
        }

        if !decel.is_empty() {
            self.sm
                .tx()
                .dma_push(self.dma.reborrow(), &decel, false)
                .await;
        }

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
