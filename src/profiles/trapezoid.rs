use crate::{
    config::{
        STEPPER_DEFAULT_ACCELERATION, STEPPER_DEFAULT_START_DELAY, STEPPER_MAX_ACCELERATION_STEPS,
    },
    profiles::MotionProfile,
};

#[derive(Debug, Clone, Copy)]
pub struct TrapezoidProfile {
    pub start_delay: u32,
    pub acceleration: u32,
}

impl MotionProfile for TrapezoidProfile {
    fn delays(
        &self,
        steps: u32,
    ) -> (
        heapless::Vec<u32, STEPPER_MAX_ACCELERATION_STEPS>,
        usize,
        heapless::Vec<u32, STEPPER_MAX_ACCELERATION_STEPS>,
    ) {
        let steps: usize = steps as usize + 1;
        let full_ramp_len = (self.start_delay / self.acceleration) as usize;
        let ramp_len = full_ramp_len.min(steps as usize / 2);
        let cruise_steps = steps as usize - 2 * ramp_len;

        let mut accel = heapless::Vec::<u32, STEPPER_MAX_ACCELERATION_STEPS>::new();
        let mut decel = heapless::Vec::<u32, STEPPER_MAX_ACCELERATION_STEPS>::new();
        for i in 0..ramp_len {
            let _ = accel.push(self.start_delay - (i as u32 * self.acceleration));
            let _ = decel.push(self.start_delay - ((ramp_len - 1 - i) as u32 * self.acceleration));
        }

        (accel, cruise_steps, decel)
    }
}

impl Default for TrapezoidProfile {
    fn default() -> Self {
        Self {
            start_delay: STEPPER_DEFAULT_START_DELAY,
            acceleration: STEPPER_DEFAULT_ACCELERATION,
        }
    }
}
