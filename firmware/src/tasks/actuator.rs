use embassy_futures::select::{Either, select};
use embassy_rp::{
    Peri,
    gpio::{Input, Level, Output, Pull},
    peripherals::{
        DMA_CH1, DMA_CH2, PIN_6, PIN_7, PIN_10, PIN_11, PIN_12, PIN_15, PIN_18, PIN_19, PIN_20, PIN_27, PIO0, PIO1
    },
    pio::{Common, Irq, StateMachine},
    pio_programs::pwm::{PioPwm, PioPwmProgram},
};
use embassy_time::Timer;

use crate::{
    COMMAND_CHANNEL, RESPONSE_CHANNEL,
    actuators::{Drivetrain, Gripper, PioStepperProgram},
    config::DrivetrainProfile,
    packet::{Action, RequestPacket, ResponsePacket},
    sensors::Proximity,
    strategy::handle_game,
};

pub type GripperType<'d> = Gripper<'d, PIO0, 2>;
pub type DrivetrainType<'d> = Drivetrain<'d, PIO1, 0, 1, DrivetrainProfile, DMA_CH1, DMA_CH2>;
pub type EnablesType<'d> = (Output<'d>, Output<'d>);

async fn wait_for_starter<'d>(starter: &mut Input<'d>) -> RequestPacket {
    starter.wait_for_rising_edge().await;
    RequestPacket::Game
}

async fn handle_action<'d>(
    action: Action,
    gripper: &mut GripperType<'d>,
    drivetrain: &mut DrivetrainType<'d>,
    proximity: &mut Proximity<'d>,
    enables: &mut EnablesType<'d>,
) {
    match action {
        Action::GripperOpen => gripper.open().await,
        Action::GripperClose => gripper.close().await,
        Action::ExtensionPush => {}
        Action::ExtensionPull => {}
        Action::Drive(distance) => {
            drivetrain.drive(distance as f64).await;
        }
        Action::Turn(degree) => {
            drivetrain.turn(degree as f64).await;
        }
        Action::SetExtensionFrequency(_freq) => {}
        Action::SetProximityEnable(en) => proximity.enable = en,
        Action::SetProximityThreshold(thres) => proximity.threshold = thres,
        Action::SetDrivetrainEnable(en) => {
            enables.0.set_level(en.into());
            enables.1.set_level(en.into());
        }
        Action::SetExtensionEnable(_en) => {}
        Action::SetColor(color) => drivetrain.set_color(color),
        Action::SetAcceleration(acceleration) => drivetrain.profile.acceleration = acceleration,
        Action::SetMaxSpeed(max_speed) => drivetrain.profile.max_speed = max_speed,
        // Action::SetPCoefficient(p) => drivetrain.profile.p = p,
    };
}

#[embassy_executor::task]
pub async fn task(
    mut common0: Common<'static, PIO0>,
    mut common1: Common<'static, PIO1>,
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
    proximity: (Peri<'static, PIN_6>, Peri<'static, PIN_7>),
    stepper1: (Peri<'static, PIN_18>, Peri<'static, PIN_20>),
    stepper2: (Peri<'static, PIN_11>, Peri<'static, PIN_12>),
    enables: (Peri<'static, PIN_19>, Peri<'static, PIN_10>),
    starter: Peri<'static, PIN_15>,
    srv2pwm: Peri<'static, PIN_27>,
) -> ! {
    let prg0 = PioPwmProgram::new(&mut common0);

    let pwm02 = PioPwm::new(&mut common0, sm02, srv2pwm, &prg0);

    let mut enables = (
        Output::new(enables.0, Level::High),
        Output::new(enables.1, Level::High),
    );

    let mut gripper: GripperType = Gripper::new(pwm02);

    let mut proximity = Proximity::new(proximity.1, proximity.0);

    let prg1 = PioStepperProgram::<_, true>::new(&mut common1);

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
        DrivetrainProfile::default(),
        &prg1,
    );

    let mut starter = Input::new(starter, Pull::Up);

    loop {
        let cmd = match select(COMMAND_CHANNEL.wait(), wait_for_starter(&mut starter)).await {
            Either::First(r) => r,
            Either::Second(r) => r,
        };

        log::info!("Actuator received: {:?}", cmd);

        match cmd {
            RequestPacket::Game => {
                select(
                    handle_game(&mut gripper, &mut drivetrain, &mut enables),
                    proximity.wait_for_proximity(),
                )
                .await;
                log::info!("END");
            }
            RequestPacket::Action(action) => {
                handle_action(
                    action,
                    &mut gripper,
                    &mut drivetrain,
                    &mut proximity,
                    &mut enables,
                )
                .await
            }
            RequestPacket::Custom(vec) => {
                for action in vec {
                    handle_action(
                        action,
                        &mut gripper,
                        &mut drivetrain,
                        &mut proximity,
                        &mut enables,
                    )
                    .await;
                }
            }
            RequestPacket::TestExtension(_number) => {}
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

        RESPONSE_CHANNEL.signal(ResponsePacket::default());
    }
}
