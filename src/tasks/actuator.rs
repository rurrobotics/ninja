use embassy_rp::{
    gpio::{Level, Output},
    peripherals::{PIN_5, PIN_6, PIN_10, PIN_15, PIN_17, PIN_18, PIN_27, PIN_28, PIO0, PIO1},
    pio::{Common, StateMachine},
    pio_programs::pwm::{PioPwm, PioPwmProgram},
};


use crate::{
    COMMAND_CHANNEL,
    actuators::{Servo, Stepper, PioStepperProgram},
    packet::StepperDirection,
};

#[embassy_executor::task]
pub async fn task(
    mut common0: Common<'static, PIO0>,
    mut common1: Common<'static, PIO1>,
    sm01: StateMachine<'static, PIO0, 1>,
    sm02: StateMachine<'static, PIO0, 2>,
    sm10: StateMachine<'static, PIO1, 0>,
    sm11: StateMachine<'static, PIO1, 1>,
    sm12: StateMachine<'static, PIO1, 2>,
    stp1dir: embassy_rp::Peri<'static, PIN_5>,
    stp1stp: embassy_rp::Peri<'static, PIN_6>,
    stp2dir: embassy_rp::Peri<'static, PIN_28>,
    stp2stp: embassy_rp::Peri<'static, PIN_27>,
    stp3dir: embassy_rp::Peri<'static, PIN_18>,
    stp3stp: embassy_rp::Peri<'static, PIN_17>,
    srv1pwm: embassy_rp::Peri<'static, PIN_10>,
    srv2pwm: embassy_rp::Peri<'static, PIN_15>,
) -> ! {
    let prg0 = PioPwmProgram::new(&mut common0);

    let pwm01 = PioPwm::new(&mut common0, sm01, srv1pwm, &prg0);
    let pwm02 = PioPwm::new(&mut common0, sm02, srv2pwm, &prg0);

    let mut servo1 = Servo::new(pwm01);
    let mut servo2 = Servo::new(pwm02);

    let prg1 = PioStepperProgram::new(&mut common1);

    let dir1out = Output::new(stp1dir, Level::Low);
    let dir2out = Output::new(stp2dir, Level::Low);
    let dir3out = Output::new(stp3dir, Level::Low);

    let mut stepper1 = Stepper::new(stp1stp, dir1out);
    let mut stepper2 = Stepper::new(stp2stp, dir2out);
    let mut stepper3 = Stepper::new(stp3stp, dir3out);

    servo1.start();
    servo2.start();

    stepper1.start();
    stepper2.start();
    stepper3.start();

    loop {
        let cmd = COMMAND_CHANNEL.receive().await;

        log::info!("Actuator received: {:?}", cmd);

        if let Some(angle) = cmd.servo1 {
            servo1.rotate(angle);
        }

        if let Some(angle) = cmd.servo2 {
            servo2.rotate(angle);
        }

        if cmd.stepper1.distance != 0 {
            stepper1.run(
                cmd.stepper1.distance,
                cmd.stepper1.direction == StepperDirection::FORWARD,
            );
        }

        if cmd.stepper2.distance != 0 {
            stepper1.run(
                cmd.stepper2.distance,
                cmd.stepper2.direction == StepperDirection::FORWARD,
            );
        }

        if cmd.stepper3.distance != 0 {
            stepper1.run(
                cmd.stepper3.distance,
                cmd.stepper2.direction == StepperDirection::FORWARD,
            );
        }
    }
}
