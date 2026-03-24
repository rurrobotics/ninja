use embassy_rp::{
    peripherals::{
        PIN_5, PIN_6, PIN_8, PIN_10, PIN_15, PIN_17, PIN_18, PIN_27, PIN_28, PIO0, PIO1,
    },
    pio::{Common, Irq, StateMachine},
    pio_programs::pwm::{PioPwm, PioPwmProgram},
};

use crate::{
    COMMAND_CHANNEL,
    actuators::{Extension, Gripper, PioStepperProgram, Stepper},
    packet::RequestPacket,
};

#[embassy_executor::task]
pub async fn task(
    mut common0: Common<'static, PIO0>,
    mut common1: Common<'static, PIO1>,
    sm01: StateMachine<'static, PIO0, 1>,
    sm02: StateMachine<'static, PIO0, 2>,
    sm_irq10: (StateMachine<'static, PIO1, 0>, Irq<'static, PIO1, 0>),
    sm_irq11: (StateMachine<'static, PIO1, 1>, Irq<'static, PIO1, 1>),
    sm_irq12: (StateMachine<'static, PIO1, 2>, Irq<'static, PIO1,2>),
    btn: embassy_rp::Peri<'static, PIN_8>,
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

    // let pwm01 = PioPwm::new(&mut common0, sm01, srv1pwm, &prg0);
    let pwm02 = PioPwm::new(&mut common0, sm02, srv2pwm, &prg0);

    // let mut servo1 = ServoBuilder::new(pwm01).build();
    let mut gripper = Gripper::new(pwm02);

    let prg1 = PioStepperProgram::new(&mut common1);

    let mut stepper1 = Stepper::new(
        &mut common1,
        sm_irq10.0,
        sm_irq10.1,
        stp1stp,
        stp1dir,
        &prg1,
    );
    let mut stepper2 = Stepper::new(
        &mut common1,
        sm_irq11.0,
        sm_irq11.1,
        stp2stp,
        stp2dir,
        &prg1,
    );
    let mut extension = Extension::new(
        &mut common1,
        sm_irq12.0,
        sm_irq12.1,
        stp3stp,
        stp3dir,
        btn,
        &prg1,
    );

    // Home
    log::info!("Homing");
    gripper.close().await;
    extension.home(&mut common1, &prg1).await;

    loop {
        let cmd = COMMAND_CHANNEL.wait().await;

        log::info!("Actuator received: {:?}", cmd);

        match cmd {
            RequestPacket::GripperOpen => gripper.open().await,
            RequestPacket::GripperClose => gripper.close().await,
            RequestPacket::ExtensionPush => extension.push().await,
            RequestPacket::ExtensionPull => extension.pull().await,
            RequestPacket::LeftStep(s) => stepper1.step(s).await,
            RequestPacket::RightStep(s) => stepper2.step(s).await,
            RequestPacket::TestExtension => loop {
                extension.push().await;
                extension.pull().await;
            },
        };
    }
}
