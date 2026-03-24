use embassy_rp::{pio::Instance, pio_programs::pwm::PioPwm};
use embassy_time::Timer;

use crate::{
    actuators::{Servo, ServoBuilder},
    config::{
        GRIPPER_ACTUATE_TIME, GRIPPER_MAX_ANGLE, GRIPPER_MAX_PULSE_WIDTH, GRIPPER_MIN_ANGLE,
        GRIPPER_MIN_PULSE_WIDTH, GRIPPER_REFRESH_INTERVAL,
    },
};

pub struct Gripper<'d, T: Instance, const SM: usize> {
    servo: Servo<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> Gripper<'d, T, SM> {
    pub fn new(pwm: PioPwm<'d, T, SM>) -> Self {
        Self {
            servo: ServoBuilder::new(pwm)
                .set_max_pulse_width(GRIPPER_MAX_PULSE_WIDTH)
                .set_min_pulse_width(GRIPPER_MIN_PULSE_WIDTH)
                .set_period(GRIPPER_REFRESH_INTERVAL)
                .build(),
        }
    }

    pub async fn open(&mut self) {
        self.servo.rotate(GRIPPER_MIN_ANGLE);
        self.servo.start();
        Timer::after(GRIPPER_ACTUATE_TIME).await;
        self.servo.stop();
    }

    pub async fn close(&mut self) {
        self.servo.rotate(GRIPPER_MAX_ANGLE);
        self.servo.start();
        Timer::after(GRIPPER_ACTUATE_TIME).await;
        self.servo.stop();
    }
}
