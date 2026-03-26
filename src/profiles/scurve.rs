use crate::{
    config::{
        STEPPER_DEFAULT_ACCELERATION, STEPPER_DEFAULT_JERK, STEPPER_DEFAULT_START_DELAY,
        STEPPER_MAX_ACCELERATION_STEPS,
    },
    profiles::MotionProfile,
};

#[derive(Debug, Clone, Copy)]
pub struct SCurveProfile {
    pub start_delay: u32,
    pub acceleration: u32,
    pub jerk: u32,
}

impl MotionProfile for SCurveProfile {
    fn delays(
        &self,
        steps: u32,
    ) -> (
        heapless::Vec<u32, STEPPER_MAX_ACCELERATION_STEPS>,
        usize,
        heapless::Vec<u32, STEPPER_MAX_ACCELERATION_STEPS>,
    ) {
        let steps = steps as usize + 1;
        let full_ramp_len = (self.start_delay / self.acceleration) as usize;
        let ramp_len = full_ramp_len.min(steps / 2);
        let cruise_steps = steps - 2 * ramp_len;

        let mut cumulative = heapless::Vec::<u32, STEPPER_MAX_ACCELERATION_STEPS>::new();
        let mut running = 0u32;
        for i in 0..ramp_len {
            let t = i as u32;
            let half = (ramp_len / 2) as u32;
            let current_accel = if t < half {
                (t * self.jerk).min(self.acceleration)
            } else {
                self.acceleration.saturating_sub((t - half) * self.jerk)
            };
            running += current_accel;
            let _ = cumulative.push(running);
        }

        let mut accel = heapless::Vec::<u32, STEPPER_MAX_ACCELERATION_STEPS>::new();
        let mut decel = heapless::Vec::<u32, STEPPER_MAX_ACCELERATION_STEPS>::new();
        for i in 0..ramp_len {
            let _ = accel.push(self.start_delay.saturating_sub(cumulative[i]));
            let _ = decel.push(
                self.start_delay
                    .saturating_sub(cumulative[ramp_len - 1 - i]),
            );
        }

        (accel, cruise_steps, decel)
    }
}

impl Default for SCurveProfile {
    fn default() -> Self {
        Self {
            start_delay: STEPPER_DEFAULT_START_DELAY,
            acceleration: STEPPER_DEFAULT_ACCELERATION,
            jerk: STEPPER_DEFAULT_JERK,
        }
    }
}
