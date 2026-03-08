use embassy_rp::{peripherals::{PIN_10, PIN_15, PWM_SLICE5, PWM_SLICE7}, pwm::{Config, Pwm}};

use crate::COMMAND_CHANNEL;

#[embassy_executor::task]
pub async fn task(
    pwm5: embassy_rp::Peri<'static, PWM_SLICE5>,
    pwm7: embassy_rp::Peri<'static, PWM_SLICE7>,
    gp10: embassy_rp::Peri<'static, PIN_10>,
    gp15: embassy_rp::Peri<'static, PIN_15>,
) -> ! {
    let mut config1 = Config::default();
    config1.top = 20000;
    config1.divider = 125.into();
    let servo1 = Pwm::new_output_a(pwm5, gp10, config1);

    let mut config2 = Config::default();
    config2.top = 20000;
    config2.divider = 125.into();
    let servo2 = Pwm::new_output_b(pwm7, gp15, config2);

    fn angle_to_duty(angle: u32) -> u16 {
        let pulse_us = 1000 + (angle.min(180) * 1000 / 180);
        pulse_us as u16
    }

    loop {
        let cmd = COMMAND_CHANNEL.receive().await;

        log::info!("Actuator received: {:?}", cmd);

        if let Some(angle) = cmd.servo1 {
            let duty = angle_to_duty(angle);
            servo1.set_counter(duty);
        }

        if let Some(angle) = cmd.servo2 {
            let duty = angle_to_duty(angle);
            servo2.set_counter(duty);
        }

        // TODO: Stepper
    }
}
