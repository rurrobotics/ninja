use core::time::Duration;
use embassy_rp::{pio::Instance, pio_programs::pwm::PioPwm};
use embassy_time::Timer;

use crate::config::{
    GRIPPER_ACTUATE_TIME, GRIPPER_MAX_ANGLE, GRIPPER_MAX_DEGREE_ROTATION, GRIPPER_MAX_PULSE, GRIPPER_MIN_ANGLE, GRIPPER_MIN_PULSE, GRIPPER_REFRESH_INTERVAL
};

pub struct Gripper<'d, T: Instance, const SM: usize> {
    pwm: PioPwm<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> Gripper<'d, T, SM> {
    pub fn new(mut pwm: PioPwm<'d, T, SM>) -> Self {
        pwm.set_period(GRIPPER_REFRESH_INTERVAL);
        Self { pwm }
    }

    fn rotate(&mut self, degree: u64) {
        let degree_per_nano_second = (GRIPPER_MAX_PULSE.as_nanos() as u64
            - GRIPPER_MIN_PULSE.as_nanos() as u64)
            / GRIPPER_MAX_DEGREE_ROTATION;
        let duration = Duration::from_nanos(
            degree * degree_per_nano_second + GRIPPER_MIN_PULSE.as_nanos() as u64,
        );

        self.pwm.write(duration);
    }

    pub async fn open(&mut self) {
        self.rotate(GRIPPER_MIN_ANGLE);
        self.pwm.start();
        Timer::after(GRIPPER_ACTUATE_TIME).await;
        self.pwm.stop();
    }

    pub async fn close(&mut self) {
        self.rotate(GRIPPER_MAX_ANGLE);
        self.pwm.start();
        Timer::after(GRIPPER_ACTUATE_TIME).await;
        self.pwm.stop();
    }
}
