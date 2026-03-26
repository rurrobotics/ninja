use embassy_futures::join::join3;
use embassy_rp::{
    Peri,
    peripherals::{
        DMA_CH1, DMA_CH2, PIN_5, PIN_6, PIN_8, PIN_10, PIN_15, PIN_17, PIN_18, PIN_27, PIN_28,
        PIO0, PIO1,
    },
    pio::{Common, Irq, StateMachine},
    pio_programs::pwm::{PioPwm, PioPwmProgram},
};
use embassy_time::Timer;

use crate::{
    COMMAND_CHANNEL,
    actuators::{Drivetrain, Extension, Gripper, PioStepperProgram},
    packet::RequestPacket,
    profiles::SCurveProfile,
};

#[embassy_executor::task]
pub async fn task(
    mut common0: Common<'static, PIO0>,
    mut common1: Common<'static, PIO1>,
    sm01: StateMachine<'static, PIO0, 1>,
    sm02: StateMachine<'static, PIO0, 2>,
    sm_irq10: (
        StateMachine<'static, PIO1, 0>,
        Irq<'static, PIO1, 0>,
        Peri<'static, DMA_CH1>,
    ),
    sm_irq11: (
        StateMachine<'static, PIO1, 1>,
        Irq<'static, PIO1, 1>,
        Peri<'static, DMA_CH2>,
    ),
    sm_irq12: (StateMachine<'static, PIO1, 2>, Irq<'static, PIO1, 2>),
    btn: embassy_rp::Peri<'static, PIN_8>,
    stp1dir: Peri<'static, PIN_5>,
    stp1stp: Peri<'static, PIN_6>,
    stp2dir: Peri<'static, PIN_28>,
    stp2stp: Peri<'static, PIN_27>,
    stp3dir: Peri<'static, PIN_18>,
    stp3stp: Peri<'static, PIN_17>,
    srv1pwm: Peri<'static, PIN_10>,
    srv2pwm: Peri<'static, PIN_15>,
) -> ! {
    let prg0 = PioPwmProgram::new(&mut common0);

    // let pwm01 = PioPwm::new(&mut common0, sm01, srv1pwm, &prg0);
    let pwm02 = PioPwm::new(&mut common0, sm02, srv2pwm, &prg0);

    // let mut servo1 = ServoBuilder::new(pwm01).build();
    let mut gripper = Gripper::new(pwm02);

    let prg1 = PioStepperProgram::<_, true>::new(&mut common1);
    let prg2 = PioStepperProgram::<_, false>::new(&mut common1);

    let mut drivetrain = Drivetrain::new(
        &mut common1,
        sm_irq10.0,
        sm_irq10.1,
        stp1stp,
        stp1dir,
        sm_irq10.2,
        sm_irq11.0,
        sm_irq11.1,
        stp2stp,
        stp2dir,
        sm_irq11.2,
        SCurveProfile::default(),
        &prg1,
    );

    let mut extension = Extension::new(
        &mut common1,
        sm_irq12.0,
        sm_irq12.1,
        stp3stp,
        stp3dir,
        btn,
        &prg2,
    );

    // Home
    log::info!("Homing");
    gripper.close().await;
    extension.home(&mut common1, &prg2).await;

    loop {
        let cmd = COMMAND_CHANNEL.wait().await;

        log::info!("Actuator received: {:?}", cmd);

        match cmd {
            RequestPacket::GripperOpen => gripper.open().await,
            RequestPacket::GripperClose => gripper.close().await,
            RequestPacket::ExtensionPush => extension.push().await,
            RequestPacket::ExtensionPull => extension.pull().await,
            RequestPacket::Drive(s) => {
                drivetrain.drive(s as f64).await;
            }
            RequestPacket::Turn(d) => {
                drivetrain.turn(d as f64).await;
            }
            RequestPacket::TestRotation => {}
            RequestPacket::TestExtension => loop {
                extension.push().await;
                Timer::after_millis(100).await;
                extension.pull().await;
                Timer::after_millis(100).await;
            },
            RequestPacket::Game => {
                join3(drivetrain.drive(200.0), gripper.open(), extension.push()).await;
                // drivetrain.turn(90.0).await;
                // drivetrain.drive(100.0).await;
                gripper.close().await;
                Timer::after_millis(200).await;
                extension.pull().await;
                Timer::after_millis(200).await;
                // drivetrain.turn(90.0).await;
                drivetrain.drive(150.0).await;
                Timer::after_millis(200).await;
                drivetrain.drive(-350.0).await;
            }
        };
    }
}
