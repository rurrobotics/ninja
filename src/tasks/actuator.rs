use embassy_rp::{
    peripherals::{PIN_10, PIN_15, PIO0},
    pio::{Common, StateMachine},
    pio_programs::pwm::{PioPwm, PioPwmProgram},
};

use crate::{COMMAND_CHANNEL, actuators::Servo};

#[embassy_executor::task]
pub async fn task(
    mut common: Common<'static, PIO0>,
    sm1: StateMachine<'static, PIO0, 1>,
    sm2: StateMachine<'static, PIO0, 2>,
    gp10: embassy_rp::Peri<'static, PIN_10>,
    gp15: embassy_rp::Peri<'static, PIN_15>,
) -> ! {
    let prg = PioPwmProgram::new(&mut common);

    let pwm1 = PioPwm::new(&mut common, sm1, gp10, &prg);
    let pwm2 = PioPwm::new(&mut common, sm2, gp15, &prg);

    let mut servo1 = Servo::new(pwm1);
    let mut servo2 = Servo::new(pwm2);

    servo1.start();
    servo2.start();

    loop {
        let cmd = COMMAND_CHANNEL.receive().await;

        log::info!("Actuator received: {:?}", cmd);

        if let Some(angle) = cmd.servo1 {
            servo1.rotate(angle);
        }

        if let Some(angle) = cmd.servo2 {
            servo2.rotate(angle);
        }

        // TODO: Stepper
    }
}
