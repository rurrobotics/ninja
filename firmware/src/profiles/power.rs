use crate::{
    config::{
        POWER_DEFAULT_ACCELERATION, POWER_DEFAULT_MAX_SPEED, POWER_DEFAULT_P,
        POWER_DEFAULT_STEPS_PER_REVOLUTION, STEPPER_WITHACC_TIMER_FREQUENCY,
    },
    profiles::{MotionProfile, Profile},
};

#[derive(Debug, Clone, Copy)]
pub struct PowerProfile<const STEPS_PER_REVOLUTION: u32 = POWER_DEFAULT_STEPS_PER_REVOLUTION> {
    pub acceleration: f64,
    pub max_speed: f64,
    pub p: f64,
}

impl<const STEPS_PER_REVOLUTION: u32> MotionProfile for PowerProfile<STEPS_PER_REVOLUTION> {
    fn delays(&self, steps: u32) -> Profile {
        let mut vec = Profile::new();

        let rad_per_step = (2.0 * core::f64::consts::PI) / STEPS_PER_REVOLUTION as f64;
        let f = STEPPER_WITHACC_TIMER_FREQUENCY as f64;
        let c0 = f * libm::sqrt(2.0 * rad_per_step / self.acceleration);

        let max_n = (self.max_speed * self.max_speed) / (2.0 * self.acceleration * rad_per_step)
            * (self.p + 2.0)
            / 2.0;
        let accel_steps = core::cmp::min(libm::floor(max_n) as u32, steps / 2);
        let coast_steps = steps.saturating_sub(2 * accel_steps);

        let mut cn = 0.676 * c0;
        let mut n = 0.0;
        let k = 4.0
            * libm::pow(
                (2.0 * rad_per_step * self.acceleration) / (self.max_speed * self.max_speed),
                self.p - 1.0,
            );

        for _ in 0..accel_steps {
            let _ = vec.push(cn as u32);
            n += 1.0;
            cn = cn - (2.0 * cn) / (4.0 * n + 1.0 + k * libm::pow(n, self.p));
        }

        for _ in 0..coast_steps {
            let _ = vec.push(cn as u32);
        }

        for i in (0..accel_steps).rev() {
            if let Some(&d) = vec.get(i as usize) {
                let _ = vec.push(d);
            }
        }

        vec
    }
}

impl Default for PowerProfile {
    fn default() -> Self {
        Self {
            acceleration: POWER_DEFAULT_ACCELERATION,
            max_speed: POWER_DEFAULT_MAX_SPEED,
            p: POWER_DEFAULT_P,
        }
    }
}
