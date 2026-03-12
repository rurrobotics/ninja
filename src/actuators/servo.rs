use core::time::Duration;
use embassy_rp::{pio::Instance, pio_programs::pwm::PioPwm};

use crate::config::{
    SERVO_MAX_DEGREE_ROTATION, SERVO_MAX_PULSE, SERVO_MIN_PULSE, SERVO_REFRESH_INTERVAL,
};

pub struct Servo<'d, T: Instance, const SM: usize> {
    pwm: PioPwm<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> Servo<'d, T, SM> {
    pub fn new(mut pwm: PioPwm<'d, T, SM>) -> Self {
        pwm.set_period(SERVO_REFRESH_INTERVAL);
        Self { pwm }
    }

    pub fn start(&mut self) {
        self.pwm.start();
    }

    pub fn stop(&mut self) {
        self.pwm.stop();
    }

    pub fn write_time(&mut self, duration: Duration) {
        self.pwm.write(duration);
    }

    pub fn rotate(&mut self, degree: u64) {
        let degree_per_nano_second = (SERVO_MAX_PULSE.as_nanos() as u64
            - SERVO_MIN_PULSE.as_nanos() as u64)
            / SERVO_MAX_DEGREE_ROTATION;
        let mut duration = Duration::from_nanos(
            degree * degree_per_nano_second + SERVO_MIN_PULSE.as_nanos() as u64,
        );
        if SERVO_MAX_PULSE < duration {
            duration = SERVO_MAX_PULSE;
        }

        self.pwm.write(duration);
    }
}
