use embassy_futures::{join::join, select::select};
use embassy_rp::{
    Peri,
    peripherals::{
        DMA_CH1, DMA_CH2, PIN_5, PIN_6, PIN_8, PIN_10, PIN_15, PIN_17, PIN_18, PIN_19, PIN_21,
        PIN_27, PIN_28, PIO0, PIO1,
    },
    pio::{Common, Irq, StateMachine},
    pio_programs::pwm::{PioPwm, PioPwmProgram},
};
use embassy_time::Timer;

use crate::{
    COMMAND_CHANNEL,
    actuators::{Drivetrain, Extension, Gripper, PioStepperProgram},
    packet::{Action, RequestPacket},
    profiles::TrapezoidProfile,
    sensors::Proximity,
};

type GripperType<'d> = Gripper<'d, PIO0, 2>;
type DrivetrainType<'d> = Drivetrain<'d, PIO1, 0, 1, TrapezoidProfile, DMA_CH1, DMA_CH2>;
type ExtensionType<'d> = Extension<'d, PIO1, 2>;

async fn handle_action<'d>(
    action: Action,
    gripper: &mut GripperType<'d>,
    extension: &mut ExtensionType<'d>,
    drivetrain: &mut DrivetrainType<'d>,
    proximity: &mut Proximity<'d>,
) {
    match action {
        Action::GripperOpen => gripper.open().await,
        Action::GripperClose => gripper.close().await,
        Action::ExtensionPush => extension.push().await,
        Action::ExtensionPull => extension.pull().await,
        Action::Drive(distance) => {
            drivetrain.drive(distance as f64).await;
        }
        Action::Turn(degree) => {
            drivetrain.turn(degree as f64).await;
        }
        Action::SetDrivetrainFrequency(freq) => drivetrain.set_frequency(freq),
        Action::SetExtensionFrequency(freq) => extension.set_frequency(freq),
        Action::SetProximityEnable(en) => proximity.enable = en,
        Action::SetProximityThreshold(thres) => proximity.threshold = thres,
    };
}

async fn handle_game<'d>(
    gripper: &mut GripperType<'d>,
    extension: &mut ExtensionType<'d>,
    drivetrain: &mut DrivetrainType<'d>,
) {
    join(drivetrain.drive(160.0), gripper.open()).await;
    drivetrain.turn(-90.0).await;
    extension.push().await;
    drivetrain.drive(180.0).await;
    gripper.close().await;
    extension.pull().await;
    drivetrain.drive(40.0).await;
    drivetrain.drive(-210.0).await;

    // Push 1
    drivetrain.turn(90.0).await;
    drivetrain.drive(240.0).await;
    drivetrain.turn(-90.0).await;
    drivetrain.drive(100.0).await;
    drivetrain.turn(90.0).await;
    drivetrain.drive(170.0).await;
    drivetrain.drive(-390.0).await;
    
    // Leave
    gripper.open().await;
    extension.push().await;
    drivetrain.drive(-90.0).await;
    drivetrain.turn(90.0).await;
    drivetrain.drive(90.0).await;
    drivetrain.turn(-90.0).await;
    drivetrain.drive(480.0).await;
    drivetrain.drive(-50.0).await;
    drivetrain.turn(-70.0).await;
    drivetrain.drive(210.0).await;
}

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
    proximity: (Peri<'static, PIN_19>, Peri<'static, PIN_21>),
    btn: Peri<'static, PIN_8>,
    stepper1: (Peri<'static, PIN_5>, Peri<'static, PIN_6>),
    stepper2: (Peri<'static, PIN_28>, Peri<'static, PIN_27>),
    stepper3: (Peri<'static, PIN_18>, Peri<'static, PIN_17>),
    srv1pwm: Peri<'static, PIN_10>,
    srv2pwm: Peri<'static, PIN_15>,
) -> ! {
    let prg0 = PioPwmProgram::new(&mut common0);

    // let pwm01 = PioPwm::new(&mut common0, sm01, srv1pwm, &prg0);
    let pwm02 = PioPwm::new(&mut common0, sm02, srv2pwm, &prg0);

    // let mut servo1 = ServoBuilder::new(pwm01).build();
    let mut gripper: GripperType = Gripper::new(pwm02);

    let mut proximity = Proximity::new(proximity.1, proximity.0);

    let prg1 = PioStepperProgram::<_, true>::new(&mut common1);
    let prg2 = PioStepperProgram::<_, false>::new(&mut common1);

    let mut drivetrain = Drivetrain::new(
        &mut common1,
        sm_irq10.0,
        sm_irq10.1,
        stepper1.1,
        stepper1.0,
        sm_irq10.2,
        sm_irq11.0,
        sm_irq11.1,
        stepper2.1,
        stepper2.0,
        sm_irq11.2,
        TrapezoidProfile::default(),
        &prg1,
    );

    let mut extension = Extension::new(
        &mut common1,
        sm_irq12.0,
        sm_irq12.1,
        stepper3.1,
        stepper3.0,
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
            RequestPacket::Game => {
                select(
                    handle_game(&mut gripper, &mut extension, &mut drivetrain),
                    proximity.wait_for_proximity(),
                )
                .await;
                log::info!("END");
            }
            RequestPacket::Action(action) => {
                handle_action(
                    action,
                    &mut gripper,
                    &mut extension,
                    &mut drivetrain,
                    &mut proximity,
                )
                .await
            }
            RequestPacket::Custom(vec) => {
                for action in vec {
                    handle_action(
                        action,
                        &mut gripper,
                        &mut extension,
                        &mut drivetrain,
                        &mut proximity,
                    )
                    .await;
                }
            }

            RequestPacket::TestExtension(number) => {
                for _ in 0..number {
                    extension.push().await;
                    Timer::after_millis(100).await;
                    extension.pull().await;
                    Timer::after_millis(100).await;
                }
            }
            RequestPacket::TestRotation(number) => {
                for _ in 0..number {
                    drivetrain.turn(360.0).await;
                    Timer::after_secs(5).await;
                }
            }
            RequestPacket::TestSquare(number, distance) => {
                for _ in 0..number {
                    for _ in 0..4 {
                        drivetrain.drive(distance as f64).await;
                        Timer::after_millis(100).await;
                        drivetrain.turn(90.0).await;
                        Timer::after_millis(100).await;
                    }
                    Timer::after_secs(5).await;
                }
            }
            RequestPacket::TestLine(number, distance) => {
                for _ in 0..number {
                    drivetrain.drive(distance as f64).await;
                    Timer::after_millis(100).await;
                    drivetrain.drive(-(distance as f64)).await;
                    Timer::after_secs(5).await;
                }
            }
        }
    }
}
