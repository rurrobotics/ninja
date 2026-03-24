use core::time::Duration;

use embassy_rp::pio::Instance;
use embassy_rp::pio_programs::pwm::PioPwm;

use crate::config::{
    SERVO_DEFAULT_MAX_PULSE_WIDTH, SERVO_DEFAULT_MIN_PULSE_WIDTH, SERVO_DEFAULT_REFRESH_INTERVAL,
    SERVO_MAX_DEGREE_ROTATION,
};

pub struct ServoBuilder<'d, T: Instance, const SM: usize> {
    pwm: PioPwm<'d, T, SM>,
    period: Duration,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
}

impl<'d, T: Instance, const SM: usize> ServoBuilder<'d, T, SM> {
    pub fn new(pwm: PioPwm<'d, T, SM>) -> Self {
        Self {
            pwm,
            period: SERVO_DEFAULT_REFRESH_INTERVAL,
            min_pulse_width: SERVO_DEFAULT_MIN_PULSE_WIDTH,
            max_pulse_width: SERVO_DEFAULT_MAX_PULSE_WIDTH,
        }
    }

    pub fn set_period(mut self, duration: Duration) -> Self {
        self.period = duration;
        self
    }

    pub fn set_min_pulse_width(mut self, duration: Duration) -> Self {
        self.min_pulse_width = duration;
        self
    }

    pub fn set_max_pulse_width(mut self, duration: Duration) -> Self {
        self.max_pulse_width = duration;
        self
    }

    pub fn build(mut self) -> Servo<'d, T, SM> {
        self.pwm.set_period(self.period);
        Servo {
            pwm: self.pwm,
            min_pulse_width: self.min_pulse_width,
            max_pulse_width: self.max_pulse_width,
        }
    }
}

pub struct Servo<'d, T: Instance, const SM: usize> {
    pwm: PioPwm<'d, T, SM>,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
}

impl<'d, T: Instance, const SM: usize> Servo<'d, T, SM> {
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
        let degree_per_nano_second = (self.max_pulse_width.as_nanos() as u64
            - self.min_pulse_width.as_nanos() as u64)
            / SERVO_MAX_DEGREE_ROTATION;
        let mut duration = Duration::from_nanos(
            degree * degree_per_nano_second + self.min_pulse_width.as_nanos() as u64,
        );
        if self.max_pulse_width < duration {
            duration = self.max_pulse_width;
        }

        self.pwm.write(duration);
    }
}
