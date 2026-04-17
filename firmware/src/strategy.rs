use embassy_time::Timer;

use crate::tasks::{DrivetrainType, EnablesType, GripperType};

pub async fn handle_game<'d>(
    gripper: &mut GripperType<'d>,
    drivetrain: &mut DrivetrainType<'d>,
    enables: &mut EnablesType<'d>,
) {
    enables.0.set_low();
    enables.1.set_low();

    Timer::after_secs(85).await;

    enables.0.set_high();
    enables.1.set_high();

    drivetrain.drive(540.0).await;
    drivetrain.turn(90.0).await;
    drivetrain.drive(600.0).await;
    drivetrain.drive(-200.0).await;
    drivetrain.turn(-45.0).await;
    drivetrain.drive(720.0).await;

    enables.0.set_low();
    enables.1.set_low();

    loop {
        gripper.close().await;
        Timer::after_millis(100).await;
        gripper.open().await;
        Timer::after_millis(100).await;
    }
}
